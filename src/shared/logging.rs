use std::fs;
use std::io::Write as _;

pub fn log_error(module: &str, error: &str) {
    drop(write_log_entry(module, error));
}

fn write_log_entry(module: &str, error: &str) -> Result<(), std::io::Error> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
    })?;

    let log_dir = home_dir.join(".cache").join("claude-statusline");
    fs::create_dir_all(&log_dir)?;

    let log_path = log_dir.join("error.log");
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let entry = format!("{timestamp} [{module}] {error}\n");

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    file.write_all(entry.as_bytes())?;

    Ok(())
}
