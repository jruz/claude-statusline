mod components;
mod effect;
mod model;
mod shared;
mod view;

use effect::execute;
use model::init;
use model::update;

const DEFAULT_API_URL: &str = "https://api.anthropic.com";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|arg| arg == "--refresh-usage") {
        return run_background_refresh();
    }

    let (mut model, mut effects) = init();
    while let Some(effect) = effects.pop() {
        if let Some(msg) = execute(&effect) {
            let (new_model, new_effects) = update(model, msg);
            model = new_model;
            effects.extend(new_effects);
        }
    }

    println!("{}", view::render(&model));
    Ok(())
}

fn cleanup_lock(lock_path: &std::path::Path) {
    if let Err(err) = std::fs::remove_file(lock_path) {
        shared::logging::log_error("usage", &format!("failed to remove lock file: {err}"));
    }
}

fn fetch_usage(access_token: &str) -> Result<crate::components::usage::Response, String> {
    let url = format!("{DEFAULT_API_URL}/api/oauth/usage");

    let response = reqwest::blocking::Client::new()
        .get(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .map_err(|err| format!("request failed: {err}"))?;

    if !response.status().is_success() {
        return Err(format!("api error: {}", response.status()));
    }

    response
        .json()
        .map_err(|err| format!("parse failed: {err}"))
}

fn read_access_token() -> Result<String, String> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .map_err(|err| format!("failed to run security command: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("keychain failed: {stderr}"));
    }

    let json_str =
        String::from_utf8(output.stdout).map_err(|err| format!("invalid utf8: {err}"))?;

    let json: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|err| format!("invalid json: {err}"))?;

    json.get("claudeAiOauth")
        .and_then(|oauth| oauth.get("accessToken"))
        .and_then(|token| token.as_str())
        .map(String::from)
        .ok_or_else(|| "missing accessToken".to_string())
}

fn run_background_refresh() -> color_eyre::Result<()> {
    use crate::components::usage;
    use chrono::Utc;

    let Some(lock_path) = usage::lock_path() else {
        shared::logging::log_error("usage", "cache directory not found");
        return Ok(());
    };

    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let Ok(_lock_file) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    else {
        return Ok(());
    };

    let access_token = match read_access_token() {
        Ok(token) => token,
        Err(err) => {
            shared::logging::log_error("usage", &format!("credentials: {err}"));
            cleanup_lock(&lock_path);
            return Ok(());
        }
    };

    let response = match fetch_usage(&access_token) {
        Ok(resp) => resp,
        Err(err) => {
            shared::logging::log_error("usage", &format!("api: {err}"));
            cleanup_lock(&lock_path);
            return Ok(());
        }
    };

    let cached = usage::Cached {
        fetched_at: Utc::now(),
        response,
    };

    let Some(cache_path) = usage::cache_path() else {
        shared::logging::log_error("usage", "cache directory not found");
        cleanup_lock(&lock_path);
        return Ok(());
    };

    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&cached)?;
    std::fs::write(&cache_path, json)?;

    cleanup_lock(&lock_path);
    Ok(())
}
