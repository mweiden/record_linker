use clap::Parser;
use env_logger;
use log;
use record_linker::file_iterator::FileIterator;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Params {
    input_glob: String,
    output_file: PathBuf,
}

#[derive(Debug)]
struct ProfilingReport {
    unique_files: u32,
    total_files: u32,
    elapsed_time_secs: f64,
    rate_rps: f64,
}

fn main() -> io::Result<()> {
    env_logger::init();
    let params = Params::parse();

    log::info!("params {:?}", params);

    // input and output
    let input_files = FileIterator::new(&params.input_glob)?;
    let mut output_file = File::create(&params.output_file)?;

    // result data structures
    let mut seen: HashSet<String> = HashSet::new();
    let mut unique_files: u32 = 0;
    let mut total_files: u32 = 0;

    // main logic
    let start = Instant::now();

    for entry_result in input_files {
        let path_buf = entry_result?;
        let input_file = File::open(path_buf)?;
        let reader = BufReader::new(input_file);

        for line_result in reader.lines() {
            let line = line_result?;
            let mut parts = line.splitn(2, ',');
            if let (Some(hash), Some(file_path)) = (parts.next(), parts.next()) {
                total_files += 1;
                if seen.insert(hash.to_owned()) {
                    writeln!(output_file, "{},{}", hash, file_path)?;
                    unique_files += 1;
                }
            } else {
                log::warn!("Malformed line: {}", line);
            }
        }
    }

    let duration = start.elapsed();

    // reporting
    log::info!(
        "profiling report: {:?}",
        ProfilingReport {
            unique_files: unique_files,
            total_files: total_files,
            elapsed_time_secs: duration.as_secs_f64(),
            rate_rps: total_files as f64 / duration.as_secs_f64(),
        }
    );

    Ok(())
}
