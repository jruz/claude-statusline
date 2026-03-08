use chrono::{DateTime, Utc};
use owo_colors::OwoColorize as _;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::shared::logging;

const CACHE_TTL_SECONDS: i64 = 3600;

#[derive(Debug, Clone)]
pub enum Channel {
    Latest,
    Next,
    Stable,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ChannelError {
    #[error("unknown channel: {0}")]
    Unknown(String),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CurrentError {
    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("parse failed")]
    ParseFailed,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum LatestError {
    #[error("cache parse: {0}")]
    CacheParse(String),
}

#[derive(Debug, Clone)]
pub struct Version {
    pub current: Option<Result<String, CurrentError>>,
    pub latest: Option<Result<String, LatestError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cached {
    pub fetched_at: DateTime<Utc>,
    pub latest_version: String,
}

pub fn channel_as_str(channel: &Channel) -> &str {
    match channel {
        Channel::Latest => "latest",
        Channel::Next => "next",
        Channel::Stable => "stable",
    }
}

pub fn channel_from_str(input: &str) -> Result<Channel, ChannelError> {
    match input {
        "latest" => Ok(Channel::Latest),
        "next" => Ok(Channel::Next),
        "stable" => Ok(Channel::Stable),
        other => Err(ChannelError::Unknown(other.to_string())),
    }
}

pub fn cache_path() -> Option<PathBuf> {
    dirs::home_dir().map(|dir| {
        dir.join(".cache")
            .join("claude-statusline")
            .join("version.json")
    })
}

pub fn lock_path() -> Option<PathBuf> {
    dirs::home_dir().map(|dir| {
        dir.join(".cache")
            .join("claude-statusline")
            .join("version.lock")
    })
}

pub fn is_lock_stale(lock_path: &std::path::Path) -> bool {
    let metadata = match std::fs::metadata(lock_path) {
        Ok(meta) => meta,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return false;
            }
            logging::log_error("version", &format!("lock metadata error: {err}"));
            return true;
        }
    };
    let modified = match metadata.modified() {
        Ok(mtime) => mtime,
        Err(err) => {
            logging::log_error("version", &format!("lock mtime error: {err}"));
            return true;
        }
    };
    let age = match std::time::SystemTime::now().duration_since(modified) {
        Ok(dur) => dur,
        Err(err) => {
            logging::log_error("version", &format!("lock age error: {err}"));
            return true;
        }
    };
    age.as_secs() >= 300
}

pub fn parse_current(result: Result<String, std::io::Error>) -> Result<String, CurrentError> {
    let output = result.map_err(|err| CurrentError::CommandFailed(err.to_string()))?;

    output
        .split_whitespace()
        .next()
        .map(String::from)
        .ok_or(CurrentError::ParseFailed)
}

pub fn parse_cache(content: &str) -> Result<String, LatestError> {
    let cached: Cached =
        serde_json::from_str(content).map_err(|err| LatestError::CacheParse(err.to_string()))?;
    Ok(cached.latest_version)
}

pub fn is_cache_stale(content: &str) -> bool {
    let cached = match serde_json::from_str::<Cached>(content) {
        Ok(data) => data,
        Err(err) => {
            logging::log_error("version", &format!("cache parse error: {err}"));
            return true;
        }
    };
    let age = Utc::now().signed_duration_since(cached.fetched_at);
    age.num_seconds() >= CACHE_TTL_SECONDS
}

fn parse_version(version_str: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version_str.split('.');

    let Ok(major) = parts.next()?.parse::<u32>() else {
        return None;
    };
    let Ok(minor) = parts.next()?.parse::<u32>() else {
        return None;
    };
    let Ok(patch) = parts.next()?.parse::<u32>() else {
        return None;
    };

    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn is_newer(latest: &str, current: &str) -> Option<bool> {
    let latest_ver = parse_version(latest)?;
    let current_ver = parse_version(current)?;
    Some(latest_ver > current_ver)
}

pub fn render(version: Option<&Version>) -> String {
    let Some(ver) = version else {
        return "Version: ...".to_string();
    };

    let Some(current_result) = &ver.current else {
        return "Version: ...".to_string();
    };

    let Ok(current) = current_result else {
        return format!("Version: {}", "err".red());
    };

    match &ver.latest {
        Some(Ok(latest)) => match is_newer(latest, current) {
            Some(true) => format!("Version: {current}{}", "\u{2191}".yellow()),
            Some(false) => format!("Version: {current}"),
            None => {
                logging::log_error(
                    "version",
                    &format!("failed to parse versions: latest={latest}, current={current}"),
                );
                format!("Version: {current}")
            }
        },
        _ => format!("Version: {current}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_current_extracts_version() {
        let result = parse_current(Ok("1.0.17 (Claude Code)".to_string()));
        assert_eq!(result.expect("should parse"), "1.0.17");
    }

    #[test]
    fn parse_current_handles_simple_version() {
        let result = parse_current(Ok("1.0.17".to_string()));
        assert_eq!(result.expect("should parse"), "1.0.17");
    }

    #[test]
    fn parse_current_error_on_empty() {
        let result = parse_current(Ok(String::new()));
        assert!(matches!(result, Err(CurrentError::ParseFailed)));
    }

    #[test]
    fn is_newer_patch_bump() {
        assert_eq!(is_newer("1.0.18", "1.0.17"), Some(true));
    }

    #[test]
    fn is_newer_minor_bump() {
        assert_eq!(is_newer("1.1.0", "1.0.17"), Some(true));
    }

    #[test]
    fn is_newer_major_bump() {
        assert_eq!(is_newer("2.0.0", "1.0.17"), Some(true));
    }

    #[test]
    fn is_newer_same_version() {
        assert_eq!(is_newer("1.0.17", "1.0.17"), Some(false));
    }

    #[test]
    fn is_newer_older_version() {
        assert_eq!(is_newer("1.0.16", "1.0.17"), Some(false));
    }

    #[test]
    fn is_newer_invalid_latest() {
        assert_eq!(is_newer("invalid", "1.0.17"), None);
    }

    #[test]
    fn is_newer_invalid_current() {
        assert_eq!(is_newer("1.0.18", "invalid"), None);
    }

    #[test]
    fn render_loading() {
        assert_eq!(render(None), "Version: ...");
    }

    #[test]
    fn render_current_loading() {
        let ver = Version {
            current: None,
            latest: Some(Ok("1.0.18".to_string())),
        };
        assert_eq!(render(Some(&ver)), "Version: ...");
    }

    #[test]
    fn render_current_only() {
        let ver = Version {
            current: Some(Ok("1.0.17".to_string())),
            latest: None,
        };
        assert_eq!(render(Some(&ver)), "Version: 1.0.17");
    }

    #[test]
    fn render_update_available() {
        let ver = Version {
            current: Some(Ok("1.0.17".to_string())),
            latest: Some(Ok("1.0.18".to_string())),
        };
        let result = render(Some(&ver));
        assert!(result.contains("Version: 1.0.17"));
        assert!(result.contains("\u{2191}"));
    }

    #[test]
    fn render_no_update() {
        let ver = Version {
            current: Some(Ok("1.0.17".to_string())),
            latest: Some(Ok("1.0.17".to_string())),
        };
        assert_eq!(render(Some(&ver)), "Version: 1.0.17");
    }

    #[test]
    fn render_error() {
        let ver = Version {
            current: Some(Err(CurrentError::ParseFailed)),
            latest: None,
        };
        let result = render(Some(&ver));
        assert!(result.contains("Version:"));
        assert!(result.contains("err"));
    }
}
