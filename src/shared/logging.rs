use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

const MAX_LOG_BYTES: u64 = 1024 * 1024;

fn log_dir() -> Result<PathBuf, std::io::Error> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
    })?;
    let dir = home_dir.join(".cache").join("claude-statusline");
    Ok(dir)
}

fn log_path() -> Result<PathBuf, std::io::Error> {
    let path = log_dir()?.join("error.log");
    Ok(path)
}

pub fn open_log_file() -> Result<fs::File, std::io::Error> {
    let dir = log_dir()?;
    fs::create_dir_all(&dir)?;

    let path = log_path()?;
    rotate_if_full(&path)?;
    fs::OpenOptions::new().create(true).append(true).open(path)
}

fn rotate_if_full(path: &Path) -> Result<(), std::io::Error> {
    if is_full(path)? {
        fs::rename(path, path.with_extension("log.1"))?;
    }
    Ok(())
}

fn is_full(path: &Path) -> Result<bool, std::io::Error> {
    if !path.exists() {
        return Ok(false);
    }
    let size = fs::metadata(path)?.len();
    Ok(size > MAX_LOG_BYTES)
}

pub fn log_error(module: &str, error: &str) {
    if let Err(err) = write_log_entry(module, error) {
        eprintln!("[{module}] logging failed: {err}");
    }
}

fn write_log_entry(module: &str, error: &str) -> Result<(), std::io::Error> {
    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z");
    let entry = format!("{timestamp} [{module}] {error}\n");

    let mut file = open_log_file()?;
    let bytes = entry.as_bytes();
    file.write_all(bytes)?;

    Ok(())
}
