use crate::components::claude_model;
use crate::components::context;
use crate::components::effort;
use crate::components::plan;
use crate::components::usage;
use crate::components::version;
use crate::effect::Effect;
use crate::effect::Msg;

pub struct Model {
    pub channel: version::Channel,
    pub claude: Option<Result<String, claude_model::Error>>,
    pub context: Option<Result<context::Context, context::Error>>,
    pub effort: Option<Result<Option<String>, effort::Error>>,
    pub plan: Option<Result<plan::Progress, plan::Error>>,
    pub usage: Option<Result<usage::Usage, usage::Error>>,
    pub version: Option<version::Version>,
}

pub fn init(channel: version::Channel) -> (Model, Vec<Effect>) {
    let model = Model {
        channel,
        claude: None,
        context: None,
        effort: None,
        plan: None,
        usage: None,
        version: None,
    };
    (
        model,
        vec![
            Effect::ReadStdin,
            Effect::ReadFile {
                path: std::path::PathBuf::from("PLAN.md"),
            },
            Effect::ReadUsageCache,
            Effect::ReadVersion,
            Effect::ReadVersionCache,
        ],
    )
}

pub fn update(model: Model, msg: Msg) -> (Model, Vec<Effect>) {
    match msg {
        Msg::CacheLoaded(result) => handle_cache_loaded(model, result),
        Msg::PlanLoaded(result) => handle_plan_loaded(model, result),
        Msg::StdinReceived(result) => handle_stdin_received(model, result),
        Msg::VersionCacheLoaded(result) => handle_version_cache_loaded(model, result),
        Msg::VersionLoaded(result) => handle_version_loaded(model, result),
    }
}

fn handle_plan_loaded(
    model: Model,
    result: Result<String, std::io::Error>,
) -> (Model, Vec<Effect>) {
    let mut effects = vec![];

    let plan_result = plan::parse(result);
    if let Err(err) = &plan_result {
        effects.push(Effect::LogError {
            module: "plan".to_owned(),
            error: err.to_string(),
        });
    }

    (
        Model {
            plan: Some(plan_result),
            ..model
        },
        effects,
    )
}

fn handle_stdin_received(
    model: Model,
    result: Result<String, std::io::Error>,
) -> (Model, Vec<Effect>) {
    let mut effects = vec![];

    let input_for_log = match &result {
        Ok(content) => content.clone(),
        Err(err) => format!("<io error: {err}>"),
    };

    let stdin_for_model = clone_stdin(&result);
    let claude_model_result = claude_model::parse(stdin_for_model);
    if let Err(err) = &claude_model_result {
        effects.push(Effect::LogError {
            module: "claude_model".to_owned(),
            error: format!("{err} | input: {input_for_log}"),
        });
    }

    let stdin_for_effort = clone_stdin(&result);
    let effort_result = effort::parse(stdin_for_effort);
    if let Err(err) = &effort_result {
        effects.push(Effect::LogError {
            module: "effort".to_owned(),
            error: format!("{err} | input: {input_for_log}"),
        });
    }

    let context_result = context::parse(result);
    if let Err(err) = &context_result {
        effects.push(Effect::LogError {
            module: "context".to_owned(),
            error: format!("{err} | input: {input_for_log}"),
        });
    }

    (
        Model {
            claude: Some(claude_model_result),
            context: Some(context_result),
            effort: Some(effort_result),
            ..model
        },
        effects,
    )
}

fn clone_stdin(result: &Result<String, std::io::Error>) -> Result<String, std::io::Error> {
    match result {
        Ok(content) => {
            let cloned = content.clone();
            Ok(cloned)
        }
        Err(io_err) => {
            let kind = io_err.kind();
            let msg = io_err.to_string();
            let cloned = std::io::Error::new(kind, msg);
            Err(cloned)
        }
    }
}

fn handle_cache_loaded(
    model: Model,
    result: Result<String, std::io::Error>,
) -> (Model, Vec<Effect>) {
    let mut effects = vec![];

    let usage_result = match result {
        Ok(content) => {
            if usage::is_cache_stale(&content) {
                effects.push(Effect::SpawnBackgroundRefresh);
            }
            usage::parse_cache(&content)
        }
        Err(err) => {
            effects.push(Effect::SpawnBackgroundRefresh);
            if err.kind() != std::io::ErrorKind::NotFound {
                effects.push(Effect::LogError {
                    module: "usage".to_owned(),
                    error: format!("cache read: {err}"),
                });
            }
            let msg = err.to_string();
            let cache_err = usage::Error::CacheRead(msg);
            Err(cache_err)
        }
    };

    if let Err(usage::Error::CacheParse(parse_err)) = &usage_result {
        effects.push(Effect::LogError {
            module: "usage".to_owned(),
            error: format!("cache parse: {parse_err}"),
        });
    }

    (
        Model {
            usage: Some(usage_result),
            ..model
        },
        effects,
    )
}

fn handle_version_loaded(
    model: Model,
    result: Result<String, std::io::Error>,
) -> (Model, Vec<Effect>) {
    let mut effects = vec![];

    let current_result = version::parse_current(result);
    if let Err(err) = &current_result {
        effects.push(Effect::LogError {
            module: "version".to_owned(),
            error: err.to_string(),
        });
    }

    let new_version = match model.version {
        Some(existing) => version::Version {
            current: Some(current_result),
            latest: existing.latest,
        },
        None => version::Version {
            current: Some(current_result),
            latest: None,
        },
    };

    (
        Model {
            version: Some(new_version),
            ..model
        },
        effects,
    )
}

fn handle_version_cache_loaded(
    model: Model,
    result: Result<String, std::io::Error>,
) -> (Model, Vec<Effect>) {
    let mut effects = vec![];

    let latest_result = match result {
        Ok(content) => {
            if version::is_cache_stale(&content) {
                effects.push(Effect::SpawnVersionRefresh {
                    channel: model.channel.clone(),
                });
            }
            let parsed = version::parse_cache(&content);
            Some(parsed)
        }
        Err(err) => {
            effects.push(Effect::SpawnVersionRefresh {
                channel: model.channel.clone(),
            });
            if err.kind() != std::io::ErrorKind::NotFound {
                effects.push(Effect::LogError {
                    module: "version".to_owned(),
                    error: format!("cache read: {err}"),
                });
            }
            None
        }
    };

    if let Some(Err(version::LatestError::CacheParse(parse_err))) = &latest_result {
        effects.push(Effect::LogError {
            module: "version".to_owned(),
            error: format!("cache parse: {parse_err}"),
        });
    }

    let new_version = match model.version {
        Some(existing) => version::Version {
            current: existing.current,
            latest: latest_result,
        },
        None => version::Version {
            current: None,
            latest: latest_result,
        },
    };

    (
        Model {
            version: Some(new_version),
            ..model
        },
        effects,
    )
}
