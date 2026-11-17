mod commands;
mod components;
mod effect;
mod env;
mod model;
mod shared;
mod utils;
mod view;

use components::version::Channel;
use effect::execute;
use model::init;
use model::update;
use utils::output::Output;

const DEFAULT_API_URL: &str = "https://api.anthropic.com";

fn parse_channel(args: &[String]) -> Result<Channel, color_eyre::Report> {
    let channel_str = args
        .iter()
        .position(|arg| arg == "--channel")
        .and_then(|idx| args.get(idx + 1));

    match channel_str {
        Some(val) => components::version::channel_from_str(val)
            .map_err(|err| color_eyre::eyre::eyre!("{err}")),
        None => Ok(Channel::Stable),
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|provider| {
            color_eyre::eyre::eyre!("failed to install rustls crypto provider: {provider:?}")
        })?;

    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|arg| arg == "--debug");
    let output = match args.get(1).map(String::as_str) {
        Some("--version") => {
            let version = commands::version::run();
            return print_output(&version);
        }
        Some("--refresh-usage") => {
            install_panic_hook();
            return run_background_refresh();
        }
        Some("--refresh-version") => {
            let channel = parse_channel(&args)?;
            return run_version_refresh(&channel);
        }
        _ => {
            let channel = parse_channel(&args)?;
            let compact_window = env::compact_window();
            let (mut model, mut effects) = init(channel, compact_window);
            while let Some(effect) = effects.pop() {
                if let Some(msg) = execute(&effect, debug) {
                    let (new_model, new_effects) = update(model, msg);
                    model = new_model;
                    effects.extend(new_effects);
                }
            }
            view::render(&model)
        }
    };

    let mut stdout = std::io::stdout();
    std::io::Write::write_fmt(&mut stdout, format_args!("{output}\n"))?;
    Ok(())
}

fn print_output(output: &Output) -> color_eyre::Result<()> {
    let mut stdout = std::io::stdout();
    let bytes = output.text.as_bytes();
    std::io::Write::write_all(&mut stdout, bytes)?;
    if !output.success {
        return Err(color_eyre::eyre::eyre!("command failed"));
    }
    Ok(())
}

fn cleanup_lock(lock_path: &std::path::Path) {
    if let Err(err) = std::fs::remove_file(lock_path) {
        shared::logging::log_error("usage", &format!("failed to remove lock file: {err}"));
    }
}

fn install_panic_hook() {
    let hook = Box::new(|panic_info: &std::panic::PanicHookInfo<'_>| {
        let message = if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
            (*msg).to_string()
        } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
            msg.clone()
        } else {
            "unknown panic".to_string()
        };

        let location = match panic_info.location() {
            Some(loc) => format!("{}:{}:{}", loc.file(), loc.line(), loc.column()),
            None => "unknown location".to_string(),
        };

        shared::logging::log_error("panic", &format!("{message} at {location}"));
    });
    std::panic::set_hook(hook);
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

    let body = response
        .text()
        .map_err(|err| format!("read body failed: {err}"))?;
    serde_json::from_str(&body).map_err(|err| format!("parse failed: {err}: {body}"))
}

fn read_access_token() -> Result<String, String> {
    if cfg!(target_os = "macos") {
        return read_access_token_from_keychain();
    }
    read_access_token_from_file()
}

fn extract_access_token(json: &serde_json::Value) -> Result<String, String> {
    json.get("claudeAiOauth")
        .and_then(|oauth| oauth.get("accessToken"))
        .and_then(|token| token.as_str())
        .map(String::from)
        .ok_or_else(|| "missing accessToken".to_string())
}

fn read_access_token_from_file() -> Result<String, String> {
    let home = dirs::home_dir().ok_or_else(|| "HOME not set".to_string())?;
    let path = home.join(".claude/.credentials.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|err| format!("invalid json: {err}"))?;
    extract_access_token(&json)
}

fn read_access_token_from_keychain() -> Result<String, String> {
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
    extract_access_token(&json)
}

fn run_background_refresh() -> color_eyre::Result<()> {
    use crate::components::usage;

    let lock_path = usage::lock_path().ok_or_else(|| {
        shared::logging::log_error("usage", "lock path not found");
        color_eyre::eyre::eyre!("lock path not found")
    })?;

    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match shared::lock::state(&lock_path) {
        shared::lock::State::Held => Ok(()),
        shared::lock::State::Stale => {
            cleanup_lock(&lock_path);
            refresh_usage(&lock_path)
        }
        shared::lock::State::Free => refresh_usage(&lock_path),
    }
}

fn refresh_usage(lock_path: &std::path::Path) -> color_eyre::Result<()> {
    use crate::components::usage;
    use chrono::Utc;

    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
        .map_err(|err| {
            shared::logging::log_error("usage", &format!("lock acquire failed: {err}"));
            err
        })?;
    drop(lock_file);

    let access_token = read_access_token().map_err(|err| {
        shared::logging::log_error("usage", &format!("credentials: {err}"));
        color_eyre::eyre::eyre!("credentials: {err}")
    })?;

    let response = fetch_usage(&access_token).map_err(|err| {
        shared::logging::log_error("usage", &format!("api: {err}"));
        color_eyre::eyre::eyre!("api: {err}")
    })?;

    let cached = usage::Cached {
        fetched_at: Utc::now(),
        response,
    };

    let cache_path = usage::cache_path().ok_or_else(|| {
        shared::logging::log_error("usage", "cache path not found");
        color_eyre::eyre::eyre!("cache path not found")
    })?;

    write_cache(&cache_path, &cached).map_err(|err| {
        shared::logging::log_error("usage", &format!("cache write: {err}"));
        color_eyre::eyre::eyre!("cache write: {err}")
    })?;

    cleanup_lock(lock_path);
    Ok(())
}

fn write_cache(cache_path: &std::path::Path, cached: &impl serde::Serialize) -> Result<(), String> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("create dir failed: {err}"))?;
    }
    let json =
        serde_json::to_string_pretty(cached).map_err(|err| format!("serialize failed: {err}"))?;
    std::fs::write(cache_path, json).map_err(|err| format!("write failed: {err}"))
}

fn run_version_refresh(channel: &Channel) -> color_eyre::Result<()> {
    use crate::components::version;

    let lock_path = version::lock_path().ok_or_else(|| {
        shared::logging::log_error("version", "lock path not found");
        color_eyre::eyre::eyre!("lock path not found")
    })?;

    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match shared::lock::state(&lock_path) {
        shared::lock::State::Held => Ok(()),
        shared::lock::State::Stale => {
            cleanup_lock(&lock_path);
            refresh_version(&lock_path, channel)
        }
        shared::lock::State::Free => refresh_version(&lock_path, channel),
    }
}

fn refresh_version(lock_path: &std::path::Path, channel: &Channel) -> color_eyre::Result<()> {
    use crate::components::version;
    use chrono::Utc;

    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
        .map_err(|err| {
            shared::logging::log_error("version", &format!("lock acquire failed: {err}"));
            err
        })?;
    drop(lock_file);

    let latest_version = fetch_latest_version(channel).map_err(|err| {
        shared::logging::log_error("version", &format!("api: {err}"));
        color_eyre::eyre::eyre!("api: {err}")
    })?;

    let cached = version::Cached {
        fetched_at: Utc::now(),
        latest_version,
    };

    let cache_path = version::cache_path().ok_or_else(|| {
        shared::logging::log_error("version", "cache path not found");
        color_eyre::eyre::eyre!("cache path not found")
    })?;

    write_cache(&cache_path, &cached).map_err(|err| {
        shared::logging::log_error("version", &format!("cache write: {err}"));
        color_eyre::eyre::eyre!("cache write: {err}")
    })?;

    cleanup_lock(lock_path);
    Ok(())
}

fn fetch_latest_version(channel: &Channel) -> Result<String, String> {
    let url = format!(
        "https://registry.npmjs.org/@anthropic-ai/claude-code/{}",
        components::version::channel_as_str(channel)
    );

    let response = reqwest::blocking::Client::new()
        .get(&url)
        .header(
            "User-Agent",
            concat!("claude-statusline/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .map_err(|err| format!("request failed: {err}"))?;

    if !response.status().is_success() {
        return Err(format!("api error: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .map_err(|err| format!("parse failed: {err}"))?;

    json.get("version")
        .and_then(|val| val.as_str())
        .map(String::from)
        .ok_or_else(|| "missing version field".to_string())
}
