use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub trait Blake3Hash {
    fn blake3(&self) -> io::Result<String>;
}

impl Blake3Hash for PathBuf {
    fn blake3(&self) -> io::Result<String> {
        // Open the file
        let mut file = File::open(self)?;

        // Initialize the hasher
        let mut hasher = blake3::Hasher::new();

        // Buffer for reading chunks (64 KiB)
        let mut buffer = [0u8; 64 * 1024];

        // Read the file and update the hash in chunks
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break; // End of file
            }
            hasher.update(&buffer[..bytes_read]);
        }

        // Get the final digest and convert it to a hexadecimal string
        Ok(hasher.finalize().to_string())
    }
}
