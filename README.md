# Record Linker

Demonstration software for the following problem:

> Deduplicate a large corpus of documents:
> - ~1PiB of documents, ~500KB each
> - documents are in cloud storage, e.g. s3, gcs
> - there are too many to fit on a single machine
> - do not use existing data processing frameworks, e.g. mapreduce, spark
> - it should run in less than a day

## Approach

The solution breaks the problem down into two steps

1. Generate a collision-resistant hash of the contents of each file
2. Gather filepaths by matching hashes, and then pick just one filepath from each collection

There are two binaaries, `hash_documents` and `dedup_hashes`, one for each step:

* `hash_documents` takes a file glob and produces (hash, filepath) key-value pairs and writes them to intermediate output files sharded by a hash prefix and run identifier
* `dedup_hashes` accepts globs of the hash files and outputs a file with the unique hashes and a single filepath for that hash (removing colliding hashes, thereby removing duplicate files)

Note we know that the hash matches must be in the the same intermediate output files because their prefixes must match in order for there to be a full hash collision.

## Benefits

### Parallelism

Data input into each stage of the pipeline can be sharded and parallelized horizontally.

You can scale the hashing step by breaking down the input corpus into separate globs (e.g. `some_dir/[a-m]*.txt,some_dir/[n-z]*.txt`) and running a separate instance of `hash_documents` on each glob. The output of `hash_documents` is sharded by the first char of the hash value: intermediate hash output files follow the format `<intermediate_output_dir>/<first_char_of_hash>_<unique_run_id>.csv`. This means there are a maximum of 36 output shards (corresponding to the chars `[a-z0-9]`). This could be easily expanded by sharding by more chars from the hash prefix.

The `dedup_hashes` stage can be parallelized by running multiple instances of the binary on sub-globs of the intermediate hash output files. The clearest way to do this would be to spin up one instance of `dedup_hashes` for each hash prefix shard (e.g. `intermediate_output_dir/a_*.csv`).

### Scalability

In experiments on an Apple M3 laptop, loading data from its SSD:

- `hash_documents` can process ~1 GiB of files / sec
- `dedup_hashes` can process hash files at ~75k rows / sec

The experiments showed that the binaries were IO bound rather than CPU bound, so we might be able to go higher provided higher-throughput IO.

#### Hypothetical scenario for 1PB of documents

Scaling invariants:
- Estimated number of files: `2.2e9`
- Formula for time of the hash generation step: `1024^5 / (num_instances * network_bandwidth_gbs * 1024^3 / 8)`
- Formula for time of the hash deduplication step: `2.2e9 / (num_instances * cores * 75000)`
- Fix [10 Gbps network bandwidth](https://docs.aws.amazon.com/ec2/latest/instancetypes/gp.html#gp_network); that's 1.34 GiB/s
- Cheapest AWS instance with sustained 10 Gbps network bandwidth is `m5.8xlarge` at $1.54 / hour

| instances | runtime (hours) |
-------------------------------
| 1         | 225.5           |
| 10        | 22.6            |
| 20        | 11.3            |
| 40        | 5.6             |

Hypothetically the scaling is linear, so the cost will be ~$345.

Additionally, we can could scale network bandwidth:

| instances | network (Gbps) | runtime (hours) | instance | cost |
------------------------------------------------------------------
| 1         | 20             | 116.8           | `m5.16xlarge` | $358 |
| 1         | 25             | 94.8            | `m5n.8xlarge` | $180 |
| 1         | 50             | 51.6            | `m5n.12xlarge` | $147 | 
| 1         | 100            | 29.9            | `m5dn.24xlarge` | $196 | 
| 2         | 50             | 25.8            | `m5n.12xlarge` | $147 | 
| 4         | 50             | 12.9            | `m5n.12xlarge` | $147 | 
| 10        | 50             | 5.2             | `m5n.12xlarge` | $147 | 
| 20        | 50             | 2.6             | `m5n.12xlarge` | $147 | 

## Development

To build the project run the following command.

```
cargo build --release
```

To generate a test corpus

```
./generate_corpus <corpus_dir>
```

To run the pipeline locally use the `run_pipeline.sh` script. Example:

```
./run_pipeline '~/Desktop/corpus/[abcd]*.txt,~/Desktop/corpus/[ef01]*.txt,~/Desktop/corpus/[2345]*.txt,~/Desktop/corpus/[6789]*.txt' ~/Desktop/unique
```

## TODOs for Production

This is a demonstration project. There would be more to do to make it production ready:

1. Enumerate files from cloud object storage instead of from the local filesystem
1. Load files from cloud object storage instead of from disk
1. Robust testing
1. Deployment configuration
1. Sharding configuration