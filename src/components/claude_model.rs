use owo_colors::OwoColorize as _;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing model field")]
    MissingField,

    #[error("Failed to parse JSON")]
    ParseJson(#[source] serde_json::Error),

    #[error("Failed to read stdin")]
    ReadStdin(#[source] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct StdinJson {
    model: Option<ModelJson>,
}

#[derive(Debug, Deserialize)]
struct ModelJson {
    display_name: String,
}

pub fn parse(result: Result<String, std::io::Error>) -> Result<String, Error> {
    let input = result.map_err(Error::ReadStdin)?;
    let parsed: StdinJson = serde_json::from_str(&input).map_err(Error::ParseJson)?;

    parsed
        .model
        .map(|model| model.display_name)
        .ok_or(Error::MissingField)
}

pub fn render(model: Option<&Result<String, Error>>) -> String {
    let value = match model {
        Some(Ok(name)) => shorten(name),
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("Model: {value}")
}

fn shorten(name: &str) -> String {
    let Some((before, after)) = name.split_once("(with ") else {
        return name.to_string();
    };
    let base = before.trim_end();
    match after.split_whitespace().next() {
        Some(capacity) => format!("{base} {capacity}"),
        None => base.to_string(),
    }
}
