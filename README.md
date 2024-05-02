# Record Linker

Demonstration software for the following problem:

> Deduplicate a large corpus of documents:
> - ~1PiB of documents, ~500KiB each
> - documents are in cloud storage, e.g. s3, gcs
> - there are too many to fit on a single machine
> - do not use existing data processing frameworks, e.g. mapreduce, spark
> - it should run in less than a day

Based on some experimentation and back of the envelope scaling calculations, this solution should be able to deduplicate 1PiB of ~500KiB documents in about 45 minutes with about $93 of cloud compute.

## Approach

The solution breaks the problem down into two steps

1. Generate a collision-resistant hash of the contents of each file
1. Gather filepaths by matching hashes, and then pick just one filepath from each collection

There are two binaaries, `hash_documents` and `dedup_hashes`, one for each step:

1. `hash_documents` takes a file glob and produces (hash, filepath) key-value pairs and writes them to intermediate output files sharded by a hash prefix and run identifier
1. `dedup_hashes` accepts globs of the hash files and outputs a file with the unique hashes and a single filepath for that hash (removing colliding hashes, thereby removing duplicate files)

Note we know that the hash matches must be in the the same intermediate output files because their prefixes must match in order for there to be a full hash collision.

For maximum throughput, `hash_documents` uses the [blake3](https://github.com/BLAKE3-team/BLAKE3) hash function, which has shown to have higher throughput than other collision-resistant hash functions (e.g. `SHA*`, `MD5`, etc).

## Benefits

### Parallelism

Data input into each stage of the pipeline can be sharded and parallelized horizontally.

You can scale the hashing step by breaking down the input corpus into separate globs (e.g. `some_dir/[a-m]*.txt,some_dir/[n-z]*.txt`) and running a separate instance of `hash_documents` on each glob. The output of `hash_documents` is sharded by the first char of the hash value: intermediate hash output files follow the format `<intermediate_output_dir>/<first_char_of_hash>_<unique_run_id>.csv`. This means there are a maximum of 36 output shards (corresponding to the chars `[a-z0-9]`). This could be easily expanded by sharding by more chars from the hash prefix.

The `dedup_hashes` stage can be parallelized by running multiple instances of the binary on sub-globs of the intermediate hash output files. The clearest way to do this would be to spin up one instance of `dedup_hashes` for each hash prefix shard (e.g. `intermediate_output_dir/a_*.csv`).

### Scalability

In experiments on an Apple M3 laptop, loading data from its SSD:

- `hash_documents` can process ~1 GiB of files / sec
- `dedup_hashes` can process hash files at ~275k rows / sec

At 1 GiB/s, the bottleneck is IO from the SSD in `hash_documents`. The path to scaling this will be to increase IO either by reducing document size or increasing bandwidth.

#### Hypothetical scenario for 1PB of documents

Back of the envelope inputs/assumptions:
- Estimated number of files: `2.2e9`
- Formula for time of the hash generation step: `1024**5 / float(num_hash_instances * network_bandwidth_gbs * 1024**3 / 8)`
- Formula for time of the hash deduplication step: `2.2e9 / float(num_dedup_instances * cores * 275000)`

Results from scaling the number of intstances and network bandwidth (instance type):

| hash_instances | dedup_instances | network (Gbps) | runtime (stg1/stg2, hours) | instance | cost |
|----------------|-----------------|----------------|----------------------------|----------|------|
| 1 | 1 | 10 | 233.02/0.07 | `m5.8xlarge` | $358.95 |
| 4 | 1 | 10 | 58.25/0.07 | `m5.8xlarge` | $358.95 |
| 16 | 1 | 10 | 14.56/0.07 | `m5.8xlarge` | $358.95 |
| 32 | 1 | 10 | 7.28/0.07 | `m5.8xlarge` | $358.95 |
| 32 | 4 | 10 | 7.28/0.02 | `m5.8xlarge` | $358.95 |
| 48 | 4 | 10 | 4.85/0.02 | `m5.8xlarge` | $358.95 |
| 1 | 1 | 20 | 116.51/0.05 | `m6g.12xlarge` | $215.63 |
| 4 | 1 | 20 | 29.13/0.05 | `m6g.12xlarge` | $215.63 |
| 16 | 1 | 20 | 7.28/0.05 | `m6g.12xlarge` | $215.63 |
| 32 | 1 | 20 | 3.64/0.05 | `m6g.12xlarge` | $215.63 |
| 32 | 4 | 20 | 3.64/0.01 | `m6g.12xlarge` | $215.63 |
| 48 | 4 | 20 | 2.43/0.01 | `m6g.12xlarge` | $215.63 |
| 1 | 1 | 50 | 46.6/0.05 | `m5zn.6xlarge` | $92.37 |
| 4 | 1 | 50 | 11.65/0.05 | `m5zn.6xlarge` | $92.37 |
| 16 | 1 | 50 | 2.91/0.05 | `m5zn.6xlarge` | $92.37 |
| 32 | 1 | 50 | 1.46/0.05 | `m5zn.6xlarge` | $92.37 |
| 32 | 4 | 50 | 1.46/0.01 | `m5zn.6xlarge` | $92.37 |
| 48 | 4 | 50 | 0.97/0.01 | `m5zn.6xlarge` | $92.37 |
| 1 | 1 | 100 | 23.3/0.05 | `m5zn.12xlarge` | $92.46 |
| 4 | 1 | 100 | 5.83/0.05 | `m5zn.12xlarge` | $92.46 |
| 16 | 1 | 100 | 1.46/0.05 | `m5zn.12xlarge` | $92.46 |
| 32 | 1 | 100 | 0.73/0.05 | `m5zn.12xlarge` | $92.46 |
| 32 | 4 | 100 | 0.73/0.01 | `m5zn.12xlarge` | $92.46 |
| 48 | 4 | 100 | 0.49/0.01 | `m5zn.12xlarge` | $92.46 |

See the `cost_estimates` script for more details.

Caveats:
- Note this assumes we can con continue to process data as quickly as it is downloaded; one option to mitigate this issue would be to run more than one `hash_documents` process per instance.

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

* Enumerate files from cloud object storage instead of from the local filesystem
* Load files from cloud object storage instead of from disk
* Robust testing
* Deployment configuration
* Sharding configuration