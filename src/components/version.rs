use chrono::{DateTime, Utc};
use owo_colors::OwoColorize as _;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CACHE_TTL_SECONDS: i64 = 3600;

#[derive(Debug, Clone)]
pub struct Version {
    pub current: Option<Result<String, CurrentError>>,
    pub latest: Option<Result<String, LatestError>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cached {
    pub fetched_at: DateTime<Utc>,
    pub latest_version: String,
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
    let Ok(metadata) = std::fs::metadata(lock_path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(age) = std::time::SystemTime::now().duration_since(modified) else {
        return false;
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
    let Ok(cached) = serde_json::from_str::<Cached>(content) else {
        return true;
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

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(latest_ver), Some(current_ver)) => latest_ver > current_ver,
        _ => false,
    }
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
        Some(Ok(latest)) if is_newer(latest, current) => {
            format!("Version: {current} {}", "\u{2191}".yellow())
        }
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
        assert!(is_newer("1.0.18", "1.0.17"));
    }

    #[test]
    fn is_newer_minor_bump() {
        assert!(is_newer("1.1.0", "1.0.17"));
    }

    #[test]
    fn is_newer_major_bump() {
        assert!(is_newer("2.0.0", "1.0.17"));
    }

    #[test]
    fn is_newer_same_version() {
        assert!(!is_newer("1.0.17", "1.0.17"));
    }

    #[test]
    fn is_newer_older_version() {
        assert!(!is_newer("1.0.16", "1.0.17"));
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
