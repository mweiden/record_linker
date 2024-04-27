use glob::glob;
use std::io;
use std::iter::Iterator;
use std::path::PathBuf;

// Custom iterator to yield file paths from a directory
pub struct FileIterator {
    entries: Box<dyn Iterator<Item = io::Result<PathBuf>>>, // Iterator over directory entries
}

impl FileIterator {
    // Constructor to create a new FileIterator for a given glob
    pub fn new(pattern: &String) -> io::Result<Self> {
        let paths = glob(pattern.as_str()).unwrap();
        let entries = paths.map(|result| match result {
            Ok(path) => Ok(path),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        });
        Ok(Self {
            entries: Box::new(entries),
        })
    }
}

impl Iterator for FileIterator {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate over directory entries
        while let Some(entry) = self.entries.next() {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        // If it's a file, return the path
                        return Some(Ok(path));
                    }
                }
                Err(err) => {
                    // Return the error if there's an issue with reading the entry
                    return Some(Err(err));
                }
            }
        }
        None // No more entries to yield
    }
}
