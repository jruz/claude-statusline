use owo_colors::OwoColorize as _;
use serde::Deserialize;

use crate::shared::progress_bar;

const THRESHOLD_RED: u8 = 70;
const THRESHOLD_ORANGE: u8 = 50;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing context_window in stdin")]
    MissingContextWindow,

    #[error("Missing context_window_size in stdin")]
    MissingWindowSize,

    #[error("Failed to parse JSON")]
    ParseJson(#[source] serde_json::Error),

    #[error("Percentage out of range")]
    PercentageRange(#[from] std::num::TryFromIntError),

    #[error("Failed to read stdin")]
    ReadStdin(#[source] std::io::Error),
}

#[derive(Debug)]
pub struct Context {
    pub percentage: u8,
    pub tokens: u64,
}

#[derive(Debug, Deserialize)]
struct StdinJson {
    context_window: Option<ContextWindowJson>,
}

#[derive(Debug, Deserialize)]
struct ContextWindowJson {
    context_window_size: Option<std::num::NonZeroU64>,
    current_usage: Option<CurrentUsageJson>,
}

#[derive(Debug, Deserialize)]
struct CurrentUsageJson {
    #[serde(rename = "cache_creation_input_tokens")]
    cache_creation: u64,
    #[serde(rename = "cache_read_input_tokens")]
    cache_read: u64,
    #[serde(rename = "input_tokens")]
    input: u64,
}

pub fn parse(result: Result<String, std::io::Error>) -> Result<Context, Error> {
    let input = result.map_err(Error::ReadStdin)?;
    let parsed: StdinJson = serde_json::from_str(&input).map_err(Error::ParseJson)?;
    let window = parsed.context_window.ok_or(Error::MissingContextWindow)?;
    let size = window.context_window_size.ok_or(Error::MissingWindowSize)?;
    let usage = window.current_usage.as_ref();
    let tokens = sum_tokens(usage);
    let percentage = tokens_to_percentage(tokens, size)?;
    Ok(Context { percentage, tokens })
}

pub fn render(context: Option<&Result<Context, Error>>) -> String {
    let value = match context {
        Some(Ok(ctx)) => {
            let percentage = f64::from(ctx.percentage);
            let bar = progress_bar::render(percentage, 10);
            let colored_bar = colorize_bar(&bar.filled, ctx.percentage);
            let tokens = format_tokens(ctx.tokens);
            format!(
                "{colored_bar}{} {:>3}% ({tokens})",
                bar.empty, ctx.percentage
            )
        }
        Some(Err(_err)) => "err".red().to_string(),
        None => "...".to_string(),
    };
    format!("Context: {value}")
}

fn sum_tokens(usage: Option<&CurrentUsageJson>) -> u64 {
    match usage {
        Some(current) => current.input + current.cache_creation + current.cache_read,
        None => 0,
    }
}

fn tokens_to_percentage(tokens: u64, size: std::num::NonZeroU64) -> Result<u8, Error> {
    let divisor = size.get();
    let half = divisor.div_euclid(2);
    let scaled = tokens.saturating_mul(100);
    let rounded = scaled.saturating_add(half).div_euclid(divisor).min(100);
    let percentage = u8::try_from(rounded)?;
    Ok(percentage)
}

fn format_tokens(tokens: u64) -> String {
    let thousands = tokens.saturating_add(500).div_euclid(1_000);
    let millions = tokens.saturating_add(500_000).div_euclid(1_000_000);
    match (tokens, thousands) {
        (0..1_000, _) => format!("{tokens}"),
        (_, 0..1_000) => format!("{thousands}k"),
        _ => format!("{millions}m"),
    }
}

fn colorize_bar(text: &str, percentage: u8) -> String {
    match percentage {
        THRESHOLD_RED.. => text.red().to_string(),
        THRESHOLD_ORANGE.. => text.yellow().to_string(),
        _ => text.green().to_string(),
    }
}
