use clap::Parser;
use record_linker::file_iterator::FileIterator;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    input_glob: String,
    output_file: PathBuf,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // input and output
    let input_files = FileIterator::new(&cli.input_glob)?;
    let mut output_file = File::create(&cli.output_file)?;

    // result data structures
    let mut results: HashMap<String, Vec<String>> = HashMap::new();
    let mut unique_files: u32 = 0;
    let mut total_files: u32 = 0;

    let start = Instant::now();

    for entry_result in input_files {
        let path_buf = entry_result?;
        let input_file = File::open(path_buf)?;
        let reader = BufReader::new(input_file);

        for line_result in reader.lines() {
            let line = line_result?;
            let entry: Vec<&str> = line.split(",").collect();
            assert!(entry.len() == 2);
            let hash = entry[0].to_owned();
            let file_path = entry[1].to_owned();
            let hash_vec = results.entry(hash).or_insert(Vec::new());
            hash_vec.push(file_path);
            total_files += 1;
        }
    }

    for (hash, hash_vec) in results {
        let line = format!("{},{}\n", hash, hash_vec.iter().next().unwrap());
        output_file.write(line.as_bytes())?;
        unique_files += 1;
    }

    let duration = start.elapsed();
    println!("dedup_hashes:");
    println!("- glob processed:\t{}", cli.input_glob);
    println!("- unique / total files:\t{}/{}", unique_files, total_files);
    println!("- Elapsed time:\t\t{:.2}s", duration.as_secs_f64());

    Ok(())
}
