use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ShardStore {
    output_files: HashMap<char, File>,
    output_dir: PathBuf,
    file_suffix: String,
}

impl ShardStore {
    pub fn new(output_dir: &String, file_suffix: String) -> Self {
        ShardStore {
            output_files: HashMap::new(),
            output_dir: PathBuf::from(output_dir),
            file_suffix: file_suffix,
        }
    }

    pub fn finalize(&mut self) -> io::Result<()> {
        for file in self.output_files.values_mut() {
            file.flush()?;
        }
        Ok(())
    }

    pub fn write(&mut self, shard: char, buf: &[u8]) -> io::Result<()> {
        let file = self.output_files.entry(shard).or_insert(
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(
                    self.output_dir
                        .clone()
                        .join(format!("{}_{}.csv", shard, self.file_suffix)),
                )?,
        );
        file.write_all(buf)?;
        Ok(())
    }
}
