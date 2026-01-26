use owo_colors::OwoColorize as _;

use crate::shared::progress_bar;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("PLAN.md not found in current directory")]
    FileNotFound,

    #[error("Failed to read PLAN.md")]
    ReadError(#[source] std::io::Error),
}

#[derive(Debug)]
pub struct Progress {
    pub completed: usize,
    pub total: usize,
}

pub fn parse(result: Result<String, std::io::Error>) -> Result<Progress, Error> {
    let content = result.map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound
        } else {
            Error::ReadError(err)
        }
    })?;

    let mut completed = 0;
    let mut total = 0;

    for line in content.lines() {
        let trimmed = line.trim_start();

        match (
            trimmed.starts_with("- [ ]"),
            trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]"),
        ) {
            (true, false) => total += 1,
            (false, true) => {
                completed += 1;
                total += 1;
            }
            (true, true) | (false, false) => {}
        }
    }

    Ok(Progress { completed, total })
}

pub fn render(plan: Option<&Result<Progress, Error>>) -> String {
    let value = match plan {
        Some(Ok(progress)) => {
            let pct: f64 = if progress.total == 0 {
                100.0
            } else {
                (progress.completed as f64 / progress.total as f64) * 100.0
            };
            format!("{} {:.0}%", progress_bar::render(pct, 10), pct)
        }
        Some(Err(Error::FileNotFound)) => "file-not-found".red().to_string(),
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("PLAN.md: {value}")
}
