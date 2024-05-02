use clap::Parser;
use env_logger;
use log;
use std::collections::HashMap;
use std::io;
use std::time::Instant;

use record_linker::file_blake3::Blake3Hash;
use record_linker::file_iterator::FileIterator;
use record_linker::file_size::Size;
use record_linker::rand::random_string;
use record_linker::shard_store::ShardStore;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Params {
    input_glob: String,
    output_dir: String,
}

#[derive(Debug)]
struct ProfilingReport {
    num_files_processed: u32,
    elapsed_time_secs: f64,
    total_processed_gb: f64,
    rate_mb_per_sec: f64,
}

fn main() -> io::Result<()> {
    env_logger::init();
    let params = Params::parse();

    // input and output
    let file_iterator = FileIterator::new(&params.input_glob)?;
    let file_suffix = random_string(6);
    let mut shard_store = ShardStore::new(&params.output_dir, file_suffix.clone());

    // result data structures
    let mut results = Vec::new();
    let mut shard_counts: HashMap<char, u32> = HashMap::new();

    // initial report
    log::info!("{:?}", params);
    log::info!("file suffix: {}", file_suffix);

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
            Err(err) => log::error!("Error reading file: {}", err),
        }
    }

    for (shard, hash, file_path) in results {
        let line = format!("{},{}\n", hash, file_path);
        shard_store.write(shard, line.as_bytes())?;
    }

    let duration = start.elapsed();

    // final reporting
    log::info!("shard counts: {:?}", shard_counts);

    log::info!(
        "{:?}",
        ProfilingReport {
            num_files_processed: total_files,
            elapsed_time_secs: duration.as_secs_f64(),
            total_processed_gb: total_bytes as f64 / 1e9,
            rate_mb_per_sec: total_bytes as f64 / duration.as_secs_f64() / 1e6,
        }
    );

    Ok(())
}
