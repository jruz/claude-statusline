use std::io::Read as _;

use crate::components::usage;
use crate::shared;

pub enum Effect {
    LogError { module: String, error: String },
    ReadFile { path: std::path::PathBuf },
    ReadStdin,
    ReadUsageCache,
    SpawnBackgroundRefresh,
}

pub enum Msg {
    CacheLoaded(Result<String, std::io::Error>),
    PlanLoaded(Result<String, std::io::Error>),
    StdinReceived(Result<String, std::io::Error>),
}

pub fn execute(effect: &Effect) -> Option<Msg> {
    match effect {
        Effect::LogError { module, error } => {
            shared::logging::log_error(module, error);
            None
        }
        Effect::ReadFile { path } => {
            let result = std::fs::read_to_string(path);
            Some(Msg::PlanLoaded(result))
        }
        Effect::ReadStdin => {
            let mut input = String::new();
            let result = std::io::stdin()
                .read_to_string(&mut input)
                .map(|_bytes_read| input);
            Some(Msg::StdinReceived(result))
        }
        Effect::ReadUsageCache => {
            let result = match usage::cache_path() {
                Some(path) => std::fs::read_to_string(path),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "cache directory not found",
                )),
            };
            Some(Msg::CacheLoaded(result))
        }
        Effect::SpawnBackgroundRefresh => {
            spawn_background_refresh();
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
        let result = std::process::Command::new(exe)
            .arg("--refresh-usage")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::from(stderr_file))
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
