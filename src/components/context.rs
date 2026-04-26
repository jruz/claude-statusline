use owo_colors::OwoColorize as _;
use serde::Deserialize;

use crate::shared::progress_bar;

const CAPACITY: u64 = 200_000;
const THRESHOLD_RED: u64 = 190_000;
const THRESHOLD_ORANGE: u64 = 180_000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse JSON")]
    ParseJson(#[source] serde_json::Error),

    #[error("Failed to read stdin")]
    ReadStdin(#[source] std::io::Error),
}

#[derive(Debug)]
pub struct Context {
    pub percentage: f64,
    pub tokens: u64,
}

#[derive(Debug, Deserialize)]
struct StdinJson {
    context_window: Option<ContextWindowJson>,
}

#[derive(Debug, Deserialize)]
struct ContextWindowJson {
    current_usage: Option<CurrentUsageJson>,
}

#[derive(Debug, Deserialize)]
struct CurrentUsageJson {
    #[serde(rename = "cache_creation_input_tokens")]
    cache_creation: Option<u64>,
    #[serde(rename = "cache_read_input_tokens")]
    cache_read: Option<u64>,
    #[serde(rename = "input_tokens")]
    input: Option<u64>,
}

pub fn parse(result: Result<String, std::io::Error>) -> Result<Context, Error> {
    let input = result.map_err(Error::ReadStdin)?;
    let parsed: StdinJson = serde_json::from_str(&input).map_err(Error::ParseJson)?;

    let usage = parsed
        .context_window
        .and_then(|cw_ref| cw_ref.current_usage);

    let input_count = usage
        .as_ref()
        .and_then(|usage_ref| usage_ref.input)
        .unwrap_or(0_u64);
    let cache_creation = usage
        .as_ref()
        .and_then(|usage_ref| usage_ref.cache_creation)
        .unwrap_or(0_u64);
    let cache_read = usage
        .as_ref()
        .and_then(|usage_ref| usage_ref.cache_read)
        .unwrap_or(0_u64);
    let tokens = input_count + cache_creation + cache_read;

    let percentage = tokens_to_percentage(tokens);

    Ok(Context { percentage, tokens })
}

pub fn render(context: Option<&Result<Context, Error>>) -> String {
    let value = match context {
        Some(Ok(ctx)) => {
            let bar = progress_bar::render(ctx.percentage, 10);
            let colored_bar = format!("{}{}", colorize_bar(&bar.filled, ctx.tokens), bar.empty);
            let text = format_tokens(ctx.tokens);
            let colored_text = colorize_text(&text, ctx.tokens);
            format!("{colored_bar} {colored_text}")
        }
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("Context: {value}")
}

fn format_tokens(tokens: u64) -> String {
    let thousands = tokens.div_euclid(1000_u64);
    format!("{thousands:>3}k")
}

fn colorize_bar(text: &str, tokens: u64) -> String {
    if tokens >= THRESHOLD_RED {
        text.red().to_string()
    } else if tokens >= THRESHOLD_ORANGE {
        text.yellow().to_string()
    } else {
        text.green().to_string()
    }
}

fn colorize_text(text: &str, tokens: u64) -> String {
    if tokens >= THRESHOLD_RED {
        text.red().to_string()
    } else if tokens >= THRESHOLD_ORANGE {
        text.yellow().to_string()
    } else {
        text.to_string()
    }
}

fn tokens_to_percentage(tokens: u64) -> f64 {
    let scaled = tokens.saturating_mul(100).div_euclid(CAPACITY);
    let clamped = u32::try_from(scaled).unwrap_or(u32::MAX);
    f64::from(clamped).min(100.0)
}
