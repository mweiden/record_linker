use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;

pub trait Blake3Hash {
    fn blake3(&self) -> io::Result<String>;
}

impl Blake3Hash for PathBuf {
    fn blake3(&self) -> io::Result<String> {
        // Open the file
        let file = File::open(self)?;

        // Create a buffered reader for efficient reading
        let mut reader = BufReader::new(file);

        // Initialize the SHA-256 hasher
        let mut hasher = blake3::Hasher::new();

        // Buffer for reading chunks
        let mut buffer = [0; 1024];

        // Read the file and update the hash in chunks
        while let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break; // End of file
            }
            hasher.update(&buffer[..bytes_read]); // Update the hash with the read bytes
        }

        // Get the final digest and convert it to a hexadecimal string
        let digest = hasher.finalize();
        Ok(digest.to_string())
    }
}
