use owo_colors::OwoColorize as _;
use serde::Deserialize;

use crate::shared::progress_bar;

const THRESHOLD_RED: f64 = 60.0;
const THRESHOLD_ORANGE: f64 = 40.0;

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
        Some(Ok(pct)) => {
            let bar = progress_bar::render(*pct, 10);
            let colored_bar = format!("{}{}", colorize_bar(&bar.filled, *pct), bar.empty);
            let text = format!("{pct:>3.0}%");
            let colored_text = colorize_text(&text, *pct);
            format!("{colored_bar} {colored_text}")
        }
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("Context: {value}")
}

fn colorize_bar(text: &str, percentage: f64) -> String {
    if percentage >= THRESHOLD_RED {
        text.red().to_string()
    } else if percentage >= THRESHOLD_ORANGE {
        text.yellow().to_string()
    } else {
        text.green().to_string()
    }
}

fn colorize_text(text: &str, percentage: f64) -> String {
    if percentage >= THRESHOLD_RED {
        text.red().to_string()
    } else if percentage >= THRESHOLD_ORANGE {
        text.yellow().to_string()
    } else {
        text.to_string()
    }
}
