use std::io::Read as _;

use color_eyre::Result;
use owo_colors::{AnsiColors, OwoColorize as _};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Missing context_window data in input")]
    MissingContextWindow,

    #[error("Missing model data in input")]
    MissingModel,

    #[error("Failed to parse JSON input")]
    ParseJson(#[source] serde_json::Error),

    #[error("Failed to read stdin")]
    ReadStdin(#[source] std::io::Error),
}

#[derive(Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct ContextWindow {
    pub used_percentage: f64,
}

#[derive(Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct Model {
    pub display_name: String,
}

#[derive(Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct StatusInput {
    pub context_window: Option<ContextWindow>,
    pub model: Option<Model>,
}

fn read_stdin() -> Result<String, Error> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(Error::ReadStdin)?;
    Ok(buffer)
}

fn parse_input(input: &str) -> Result<StatusInput, Error> {
    serde_json::from_str(input).map_err(Error::ParseJson)
}

fn select_color(percentage: f64) -> AnsiColors {
    if percentage < 50.0 {
        AnsiColors::Green
    } else if percentage < 75.0 {
        AnsiColors::Yellow
    } else {
        AnsiColors::Red
    }
}

fn render_progress_bar(percentage: f64, width: u8) -> String {
    let clamped = percentage.clamp(0.0, 100.0);
    let width_usize = usize::from(width);
    let thresholds: [f64; 10] = [5.0, 15.0, 25.0, 35.0, 45.0, 55.0, 65.0, 75.0, 85.0, 95.0];
    let filled_count = thresholds
        .iter()
        .take(width_usize)
        .filter(|&&threshold| clamped >= threshold)
        .count();
    let empty_count = width_usize.saturating_sub(filled_count);

    let filled = "\u{2588}".repeat(filled_count);
    let empty = "\u{2591}".repeat(empty_count);
    let color = select_color(percentage);

    format!("{}{}", filled.color(color), empty)
}

fn format_statusline(input: &StatusInput) -> Result<String, Error> {
    let model = input.model.as_ref().ok_or(Error::MissingModel)?;
    let context = input
        .context_window
        .as_ref()
        .ok_or(Error::MissingContextWindow)?;

    let bar = render_progress_bar(context.used_percentage, 10);
    let percentage = format!("{:.0}%", context.used_percentage);

    Ok(format!(
        "[{}] \u{258c}{}\u{2590} {}",
        model.display_name, bar, percentage
    ))
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let input_str = read_stdin()?;
    let input = parse_input(&input_str)?;
    let output = format_statusline(&input)?;

    println!("{output}");
    Ok(())
}
