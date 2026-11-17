use chrono::{DateTime, Utc};
use num_traits::ToPrimitive as _;
use owo_colors::OwoColorize as _;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::shared::logging;
use crate::shared::progress_bar;

const THRESHOLD_RED: u8 = 20;
const THRESHOLD_YELLOW: u8 = 40;

#[derive(Debug, Clone)]
pub struct Usage {
    pub five_hour: Result<Window, Error>,
    pub seven_day: Result<Window, Error>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub resets_at: DateTime<Utc>,
    pub utilization: f64,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("cache parse: {0}")]
    CacheParse(String),

    #[error("cache read: {0}")]
    CacheRead(String),

    #[error("expired at {0}")]
    Expired(DateTime<Utc>),

    #[error("invalid utilization: {0}")]
    InvalidUtilization(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cached {
    pub fetched_at: DateTime<Utc>,
    pub response: Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub five_hour: Window,
    pub seven_day: Window,
}

pub fn cache_path() -> Option<PathBuf> {
    dirs::home_dir().map(|dir| {
        dir.join(".cache")
            .join("claude-statusline")
            .join("usage.json")
    })
}

pub fn lock_path() -> Option<PathBuf> {
    dirs::home_dir().map(|dir| {
        dir.join(".cache")
            .join("claude-statusline")
            .join("usage.lock")
    })
}

pub fn parse_cache(content: &str) -> Result<Usage, Error> {
    let cached: Cached = serde_json::from_str(content).map_err(|err| {
        let msg = err.to_string();
        Error::CacheParse(msg)
    })?;

    Ok(Usage {
        five_hour: reject_expired(cached.response.five_hour),
        seven_day: reject_expired(cached.response.seven_day),
    })
}

fn reject_expired(window: Window) -> Result<Window, Error> {
    if window.resets_at <= Utc::now() {
        return Err(Error::Expired(window.resets_at));
    }
    Ok(window)
}

pub fn is_cache_stale(content: &str) -> bool {
    let cached = match serde_json::from_str::<Cached>(content) {
        Ok(data) => data,
        Err(err) => {
            logging::log_error("usage", &format!("cache parse error: {err}"));
            return true;
        }
    };
    let age = Utc::now().signed_duration_since(cached.fetched_at);
    age.num_seconds() >= 60
}

fn format_remaining(resets_at: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = resets_at.signed_duration_since(now);

    if duration.num_seconds() <= 0 {
        return "0m".to_string();
    }

    let total_minutes = duration.num_minutes();
    let total_hours = duration.num_hours();
    let total_days = duration.num_days();

    if total_hours < 1 {
        format!("{total_minutes}m")
    } else if total_hours < 24 {
        let remaining_minutes = total_minutes.rem_euclid(60);
        format!("{total_hours}h {remaining_minutes}m")
    } else {
        let remaining_hours = total_hours.rem_euclid(24);
        format!("{total_days}d {remaining_hours}h")
    }
}

pub fn render(usage: Option<&Result<Usage, Error>>) -> String {
    match usage {
        Some(Ok(data)) => format!(
            "{}\n{}",
            render_window("5h", &data.five_hour),
            render_window("7d", &data.seven_day)
        ),
        Some(Err(_err)) => {
            format!("5h: {}\n7d: {}", "err".red(), "err".red())
        }
        None => "5h: ...\n7d: ...".to_string(),
    }
}

fn remaining_percentage(window: &Window) -> Result<u8, Error> {
    let invalid = Error::InvalidUtilization(window.utilization);
    let used = window
        .utilization
        .clamp(0.0, 100.0)
        .round()
        .to_u8()
        .ok_or(invalid)?;
    let remaining = 100_u8.saturating_sub(used);
    Ok(remaining)
}

fn colorize_bar(text: &str, remaining: u8) -> String {
    match remaining {
        0..THRESHOLD_RED => text.red().to_string(),
        THRESHOLD_RED..=THRESHOLD_YELLOW => text.yellow().to_string(),
        _ => text.green().to_string(),
    }
}

fn render_bar(label: &str, window: &Window) -> Result<String, Error> {
    let remaining = remaining_percentage(window)?;
    let percentage = f64::from(remaining);
    let bar = progress_bar::render(percentage, 10);
    Ok(format!(
        "{label}: {}{} {remaining:>3}% ({})",
        colorize_bar(&bar.filled, remaining),
        bar.empty,
        format_remaining(window.resets_at)
    ))
}

fn render_window(label: &str, window: &Result<Window, Error>) -> String {
    match window
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|data| render_bar(label, data))
    {
        Ok(line) => line,
        Err(err) => {
            logging::log_error("usage", &format!("{label} render: {err}"));
            format!("{label}: {}", "err".red())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_remaining_minutes() {
        let resets_at = Utc::now() + chrono::Duration::minutes(45);
        let result = format_remaining(resets_at);
        assert!(result.ends_with('m'));
        assert!(!result.contains('h'));
    }

    #[test]
    fn format_remaining_hours() {
        let resets_at = Utc::now() + chrono::Duration::hours(2) + chrono::Duration::minutes(15);
        let result = format_remaining(resets_at);
        assert!(result.contains("h "));
        assert!(result.contains('m'));
    }

    #[test]
    fn format_remaining_days() {
        let resets_at = Utc::now() + chrono::Duration::days(3) + chrono::Duration::hours(12);
        let result = format_remaining(resets_at);
        assert!(result.contains("d "));
        assert!(result.contains('h'));
    }

    fn cache_json(
        five_hour_resets_at: DateTime<Utc>,
        seven_day_resets_at: DateTime<Utc>,
    ) -> String {
        format!(
            r#"{{
            "fetched_at": "{}",
            "response": {{
                "five_hour": {{ "utilization": 40.0, "resets_at": "{}" }},
                "seven_day": {{ "utilization": 60.0, "resets_at": "{}" }}
            }}
        }}"#,
            Utc::now().to_rfc3339(),
            five_hour_resets_at.to_rfc3339(),
            seven_day_resets_at.to_rfc3339()
        )
    }

    #[test]
    fn parse_cache_valid() {
        let json = cache_json(
            Utc::now() + chrono::Duration::hours(3),
            Utc::now() + chrono::Duration::days(5),
        );

        let usage = parse_cache(&json).expect("parse should succeed");
        usage.five_hour.expect("five_hour should be ok");
        usage.seven_day.expect("seven_day should be ok");
    }

    #[test]
    fn parse_cache_rejects_expired_window() {
        let json = cache_json(
            Utc::now() - chrono::Duration::hours(1),
            Utc::now() + chrono::Duration::days(5),
        );

        let usage = parse_cache(&json).expect("parse should succeed");
        assert!(matches!(usage.five_hour, Err(Error::Expired(_))));
        usage.seven_day.expect("seven_day should be ok");
    }

    #[test]
    fn parse_api_response_matches_fixture() {
        let api_response = include_str!("../../tests/fixtures/usage_api_response.json");
        let response: Response =
            serde_json::from_str(api_response).expect("fixture should parse as Response");

        assert_eq!(response.five_hour.utilization.to_i64(), Some(95));
        assert_eq!(response.seven_day.utilization.to_i64(), Some(13));
    }
}
