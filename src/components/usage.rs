use chrono::{DateTime, Utc};
use owo_colors::OwoColorize as _;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::shared::logging;
use crate::shared::progress_bar;

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

pub fn is_lock_stale(lock_path: &std::path::Path) -> bool {
    let metadata = match std::fs::metadata(lock_path) {
        Ok(meta) => meta,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return false;
            }
            logging::log_error("usage", &format!("lock metadata error: {err}"));
            return true;
        }
    };
    let modified = match metadata.modified() {
        Ok(mtime) => mtime,
        Err(err) => {
            logging::log_error("usage", &format!("lock mtime error: {err}"));
            return true;
        }
    };
    let age = match std::time::SystemTime::now().duration_since(modified) {
        Ok(dur) => dur,
        Err(err) => {
            logging::log_error("usage", &format!("lock age error: {err}"));
            return true;
        }
    };
    age.as_secs() >= 300
}

pub fn parse_cache(content: &str) -> Result<Usage, Error> {
    let cached: Cached = serde_json::from_str(content).map_err(|err| {
        let msg = err.to_string();
        Error::CacheParse(msg)
    })?;

    Ok(Usage {
        five_hour: Ok(cached.response.five_hour),
        seven_day: Ok(cached.response.seven_day),
    })
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
        Some(Ok(data)) => format!("{}\n{}", render_five_hour(data), render_seven_day(data)),
        Some(Err(_err)) => {
            format!("5h: {}\n7d: {}", "err".red(), "err".red())
        }
        None => "5h: ...\n7d: ...".to_string(),
    }
}

fn render_five_hour(usage: &Usage) -> String {
    match &usage.five_hour {
        Ok(window) => {
            let bar = progress_bar::render(window.utilization, 10);
            format!(
                "5h: {}{} {:>3.0}% ({})",
                bar.filled.green(),
                bar.empty,
                window.utilization,
                format_remaining(window.resets_at)
            )
        }
        Err(err) => {
            logging::log_error("usage", &format!("5h render: {err}"));
            format!("5h: {}", "err".red())
        }
    }
}

fn render_seven_day(usage: &Usage) -> String {
    match &usage.seven_day {
        Ok(window) => {
            let bar = progress_bar::render(window.utilization, 10);
            format!(
                "7d: {}{} {:>3.0}% ({})",
                bar.filled.green(),
                bar.empty,
                window.utilization,
                format_remaining(window.resets_at)
            )
        }
        Err(err) => {
            logging::log_error("usage", &format!("7d render: {err}"));
            format!("7d: {}", "err".red())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::ToPrimitive as _;

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

    #[test]
    fn parse_cache_valid() {
        let json = r#"{
            "fetched_at": "2026-01-25T12:30:00Z",
            "response": {
                "five_hour": { "utilization": 0.6, "resets_at": "2026-01-25T15:30:00Z" },
                "seven_day": { "utilization": 0.4, "resets_at": "2026-01-30T00:00:00Z" }
            }
        }"#;

        let result = parse_cache(json);
        assert!(result.is_ok());
        let usage = result.expect("parse should succeed");
        usage.five_hour.expect("five_hour should be ok");
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

    #[test]
    fn is_lock_stale_returns_true_for_old_lock() {
        use std::fs::File;

        let temp_dir = std::env::temp_dir().join("claude-statusline-test");
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");
        let lock_path = temp_dir.join("test.lock");

        let lock_file = File::create(&lock_path).expect("create lock file");
        drop(lock_file);

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time since epoch")
            .as_secs();
        let past_secs = i64::try_from(now_secs).expect("secs to i64") - 600;

        let mtime = filetime::FileTime::from_unix_time(past_secs, 0);
        filetime::set_file_mtime(&lock_path, mtime).expect("set mtime");

        assert!(is_lock_stale(&lock_path));

        std::fs::remove_file(&lock_path).expect("cleanup test lock file");
    }
}
