use std::fs;
use std::io::Write as _;

use chrono;
use dirs;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cache directory not found")]
    CacheDirNotFound,

    #[error("Failed to write log entry")]
    WriteEntry(#[source] std::io::Error),
}

pub fn log_error(module: &str, error: &str) {
    let result = write_log_entry(module, error);
    drop(result);
}

fn write_log_entry(module: &str, error: &str) -> Result<(), Error> {
    let cache_dir = dirs::cache_dir().ok_or(Error::CacheDirNotFound)?;

    let log_dir = cache_dir.join("claude-statusline");
    fs::create_dir_all(&log_dir).map_err(Error::WriteEntry)?;

    let log_path = log_dir.join("error.log");
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let entry = format!("{timestamp} [{module}] {error}\n");

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(Error::WriteEntry)?;

    file.write_all(entry.as_bytes())
        .map_err(Error::WriteEntry)?;

    Ok(())
}
