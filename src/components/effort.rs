use owo_colors::OwoColorize as _;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse JSON")]
    ParseJson(#[source] serde_json::Error),

    #[error("Failed to read stdin")]
    ReadStdin(#[source] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct StdinJson {
    effort: Option<EffortJson>,
}

#[derive(Debug, Deserialize)]
struct EffortJson {
    level: String,
}

pub fn parse(result: Result<String, std::io::Error>) -> Result<Option<String>, Error> {
    let input = result.map_err(Error::ReadStdin)?;
    let parsed: StdinJson = serde_json::from_str(&input).map_err(Error::ParseJson)?;
    let level = parsed.effort.map(|effort| effort.level);
    Ok(level)
}

pub fn render(effort: Option<&Result<Option<String>, Error>>) -> String {
    let value = match effort {
        Some(Ok(Some(level))) => level.clone(),
        Some(Ok(None)) => "n/a".dimmed().to_string(),
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("Thinking: {value}")
}
