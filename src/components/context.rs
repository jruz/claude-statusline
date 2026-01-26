use owo_colors::OwoColorize as _;
use serde::Deserialize;

use crate::shared::progress_bar;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse JSON")]
    ParseJson(#[source] serde_json::Error),

    #[error("Failed to read stdin")]
    ReadStdin(#[source] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct StdinJson {
    context_window: Option<ContextWindowJson>,
}

#[derive(Debug, Deserialize)]
struct ContextWindowJson {
    used_percentage: Option<f64>,
}

pub fn parse(result: Result<String, std::io::Error>) -> Result<f64, Error> {
    let input = result.map_err(Error::ReadStdin)?;
    let parsed: StdinJson = serde_json::from_str(&input).map_err(Error::ParseJson)?;

    let default: f64 = 0.0;
    let percentage = match parsed.context_window {
        Some(ContextWindowJson {
            used_percentage: Some(pct),
        }) => pct,
        Some(ContextWindowJson {
            used_percentage: None,
        })
        | None => default,
    };

    Ok(percentage)
}

pub fn render(context: Option<&Result<f64, Error>>) -> String {
    let value = match context {
        Some(Ok(pct)) => format!("{} {:>3.0}%", progress_bar::render(*pct, 10), pct),
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("Context: {value}")
}
