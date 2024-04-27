use clap::Parser;
use std::collections::HashMap;
use std::io;
use std::time::Instant;

use record_linker::file_blake3::Blake3Hash;
use record_linker::file_iterator::FileIterator;
use record_linker::file_size::Size;
use record_linker::rand::random_string;
use record_linker::shard_store::ShardStore;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    input_glob: String,
    output_dir: String,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // input and output
    let file_iterator = FileIterator::new(&cli.input_glob)?;
    let file_suffix = random_string(6);
    let mut shard_store = ShardStore::new(&cli.output_dir, file_suffix.clone());

    // result data structures
    let mut results = Vec::new();
    let mut shard_counts: HashMap<char, u32> = HashMap::new();

    // initial report
    println!("hash_documents:");
    println!("- input glob: {}", cli.input_glob);
    println!("- ouput directory: {}", cli.output_dir);
    println!("- file suffix: {}", file_suffix);

    // main logic
    let start = Instant::now();

    let mut total_bytes: u64 = 0;
    let mut total_files: u32 = 0;

    for file in file_iterator {
        match file {
            Ok(path) => {
                let hash = path.blake3()?;
                let shard = hash.chars().next().unwrap();
                let path_str = path.to_str().unwrap().to_owned();

                results.push((shard, hash, path_str));

                let count = shard_counts.entry(shard).or_insert(0);
                *count += 1;

                total_bytes += path.size()?;
                total_files += 1;
            }
            Err(err) => eprintln!("Error reading file: {}", err),
        }
    }

    for (shard, hash, file_path) in results {
        let line = format!("{},{}\n", hash, file_path);
        shard_store.write(shard, line.as_bytes())?;
    }

    let duration = start.elapsed();

    // final reporting
    println!("\nshard\tcount");
    for (shard, count) in shard_counts {
        println!("{}\t{}", shard, count)
    }

    println!("\nProfiling:");
    println!("- Number of files processed:\t{}", total_files);
    println!("- Elapsed time:\t\t\t{:.2}s", duration.as_secs_f64());
    println!(
        "- Size of files processed:\t{:.2} GB",
        total_bytes as f64 / 1e9
    );
    println!(
        "- Processing rate:\t\t{:.2} MB/sec",
        total_bytes as f64 / duration.as_secs_f64() / 1e6
    );

    Ok(())
}
