use std::fs;
use std::io;
use std::path::PathBuf;

pub trait Size {
    fn size(&self) -> io::Result<u64>;
}

impl Size for PathBuf {
    fn size(&self) -> io::Result<u64> {
        let metadata = fs::metadata(self)?;
        Ok(metadata.len())
    }
}
