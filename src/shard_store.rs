use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ShardStore {
    output_files: HashMap<char, BufWriter<File>>,
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
        for writer in self.output_files.values_mut() {
            writer.flush()?;
        }
        Ok(())
    }

    pub fn write(&mut self, shard: char, buf: &[u8]) -> io::Result<()> {
        if !self.output_files.contains_key(&shard) {
            let path = self
                .output_dir
                .clone()
                .join(format!("{}_{}.csv", shard, self.file_suffix));
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)?;
            self.output_files.insert(shard, BufWriter::new(file));
        }
        let writer = self
            .output_files
            .get_mut(&shard)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "shard not initialized"))?;
        writer.write_all(buf)?;
        Ok(())
    }
}
