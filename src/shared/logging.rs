use std::fs;
use std::io::Write as _;
use std::path::PathBuf;

fn log_dir() -> Result<PathBuf, std::io::Error> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
    })?;
    Ok(home_dir.join(".cache").join("claude-statusline"))
}

fn log_path() -> Result<PathBuf, std::io::Error> {
    Ok(log_dir()?.join("error.log"))
}

pub fn open_log_file() -> Result<fs::File, std::io::Error> {
    let dir = log_dir()?;
    fs::create_dir_all(&dir)?;

    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path()?)
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
    file.write_all(entry.as_bytes())?;

    Ok(())
}
