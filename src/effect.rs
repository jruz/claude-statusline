use std::io::Read as _;

use crate::components::usage;
use crate::components::version;
use crate::shared;

pub enum Effect {
    LogError { module: String, error: String },
    ReadFile { path: std::path::PathBuf },
    ReadStdin,
    ReadUsageCache,
    ReadVersion,
    ReadVersionCache,
    SpawnBackgroundRefresh,
    SpawnVersionRefresh { channel: version::Channel },
}

pub enum Msg {
    CacheLoaded(Result<String, std::io::Error>),
    PlanLoaded(Result<String, std::io::Error>),
    StdinReceived(Result<String, std::io::Error>),
    VersionCacheLoaded(Result<String, std::io::Error>),
    VersionLoaded(Result<String, std::io::Error>),
}

pub fn execute(effect: &Effect) -> Option<Msg> {
    match effect {
        Effect::LogError { module, error } => {
            shared::logging::log_error(module, error);
            None
        }
        Effect::ReadFile { path } => {
            let result = std::fs::read_to_string(path);
            let msg = Msg::PlanLoaded(result);
            Some(msg)
        }
        Effect::ReadStdin => {
            let mut input = String::new();
            let result = std::io::stdin()
                .read_to_string(&mut input)
                .map(|_bytes_read| input);
            let msg = Msg::StdinReceived(result);
            Some(msg)
        }
        Effect::ReadUsageCache => {
            let result = if let Some(path) = usage::cache_path() {
                std::fs::read_to_string(path)
            } else {
                let err =
                    std::io::Error::new(std::io::ErrorKind::NotFound, "cache directory not found");
                Err(err)
            };
            let msg = Msg::CacheLoaded(result);
            Some(msg)
        }
        Effect::SpawnBackgroundRefresh => {
            spawn_background_refresh();
            None
        }
        Effect::ReadVersion => {
            let result = std::process::Command::new("claude")
                .arg("--version")
                .output()
                .map_err(|err| {
                    let msg = err.to_string();
                    std::io::Error::other(msg)
                })
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).map_err(|err| {
                            let msg = err.to_string();
                            std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
                        })
                    } else {
                        let err = std::io::Error::other("command failed");
                        Err(err)
                    }
                });
            let msg = Msg::VersionLoaded(result);
            Some(msg)
        }
        Effect::ReadVersionCache => {
            let result = if let Some(path) = version::cache_path() {
                std::fs::read_to_string(path)
            } else {
                let err =
                    std::io::Error::new(std::io::ErrorKind::NotFound, "cache directory not found");
                Err(err)
            };
            let msg = Msg::VersionCacheLoaded(result);
            Some(msg)
        }
        Effect::SpawnVersionRefresh { channel } => {
            spawn_version_refresh(channel);
            None
        }
    }
}

fn spawn_background_refresh() {
    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => {
            shared::logging::log_error("usage", &format!("failed to get current exe: {err}"));
            return;
        }
    };

    let stderr_file = match shared::logging::open_log_file() {
        Ok(file) => file,
        Err(err) => {
            shared::logging::log_error("usage", &format!("failed to open log file: {err}"));
            return;
        }
    };

    std::thread::spawn(move || {
        let null_stdin = std::process::Stdio::null();
        let null_stdout = std::process::Stdio::null();
        let stderr_stdio = std::process::Stdio::from(stderr_file);
        let result = std::process::Command::new(exe)
            .arg("--refresh-usage")
            .stdin(null_stdin)
            .stdout(null_stdout)
            .stderr(stderr_stdio)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    shared::logging::log_error("usage", "refresh completed successfully");
                } else {
                    shared::logging::log_error(
                        "usage",
                        &format!("refresh failed with status: {}", output.status),
                    );
                }
            }
            Err(err) => {
                shared::logging::log_error(
                    "usage",
                    &format!("failed to run background refresh: {err}"),
                );
            }
        }
    });
}

fn spawn_version_refresh(channel: &version::Channel) {
    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => {
            shared::logging::log_error("version", &format!("failed to get current exe: {err}"));
            return;
        }
    };

    let stderr_file = match shared::logging::open_log_file() {
        Ok(file) => file,
        Err(err) => {
            shared::logging::log_error("version", &format!("failed to open log file: {err}"));
            return;
        }
    };

    let channel_str = version::channel_as_str(channel).to_string();
    std::thread::spawn(move || {
        let null_stdin = std::process::Stdio::null();
        let null_stdout = std::process::Stdio::null();
        let stderr_stdio = std::process::Stdio::from(stderr_file);
        let result = std::process::Command::new(exe)
            .args(["--refresh-version", "--channel", &channel_str])
            .stdin(null_stdin)
            .stdout(null_stdout)
            .stderr(stderr_stdio)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    shared::logging::log_error("version", "refresh completed successfully");
                } else {
                    shared::logging::log_error(
                        "version",
                        &format!("refresh failed with status: {}", output.status),
                    );
                }
            }
            Err(err) => {
                shared::logging::log_error(
                    "version",
                    &format!("failed to run version refresh: {err}"),
                );
            }
        }
    });
}
