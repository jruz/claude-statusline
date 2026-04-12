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
    pub completed: u16,
    pub total: u16,
}

pub fn parse(result: Result<String, std::io::Error>) -> Result<Progress, Error> {
    let content = result.map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound
        } else {
            Error::ReadError(err)
        }
    })?;

    let mut completed = 0_u16;
    let mut total = 0_u16;

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
            let pct = if progress.total == 0 {
                100.0_f64
            } else {
                let completed = u32::from(progress.completed);
                let total = u32::from(progress.total);
                let raw = (completed * 100).div_euclid(total).min(100);
                f64::from(raw)
            };
            let bar = progress_bar::render(pct, 10);
            format!("{}{} {:>3.0}%", bar.filled.green(), bar.empty, pct)
        }
        Some(Err(Error::FileNotFound)) => "file-not-found".red().to_string(),
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("PLAN.md: {value}")
}
