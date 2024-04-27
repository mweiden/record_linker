use std::io;
use std::path::PathBuf;

pub fn validate_dir(path: &PathBuf) -> io::Result<()> {
    if !(path.exists() && path.is_dir()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Directory does not exist: {:?}", path),
        ));
    }
    Ok(())
}
