use std::io::Read as _;

use crate::shared;

pub enum Effect {
    LogError { module: String, error: String },
    ReadFile { path: std::path::PathBuf },
    ReadStdin,
}

pub enum Msg {
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
            let result = std::io::stdin().read_to_string(&mut input).map(|_| input);
            Some(Msg::StdinReceived(result))
        }
    }
}
