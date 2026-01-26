use crate::components::claude_model;
use crate::components::context;
use crate::components::plan;
use crate::components::usage;
use crate::effect::Effect;
use crate::effect::Msg;

pub struct Model {
    pub claude: Option<Result<String, claude_model::Error>>,
    pub context: Option<Result<f64, context::Error>>,
    pub plan: Option<Result<plan::Progress, plan::Error>>,
    pub usage: Option<Result<usage::Usage, usage::Error>>,
}

pub fn init() -> (Model, Vec<Effect>) {
    let model = Model {
        claude: None,
        context: None,
        plan: None,
        usage: None,
    };
    (
        model,
        vec![
            Effect::ReadStdin,
            Effect::ReadFile {
                path: std::path::PathBuf::from("PLAN.md"),
            },
            Effect::ReadUsageCache,
        ],
    )
}

pub fn update(model: Model, msg: Msg) -> (Model, Vec<Effect>) {
    match msg {
        Msg::PlanLoaded(result) => {
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
        Msg::StdinReceived(result) => {
            let mut effects = vec![];

            let input_for_log = result
                .as_ref()
                .map_or_else(|err| format!("<io error: {err}>"), Clone::clone);

            let claude_model_result = claude_model::parse(
                result
                    .as_ref()
                    .map(std::clone::Clone::clone)
                    .map_err(|io_err| std::io::Error::new(io_err.kind(), io_err.to_string())),
            );
            if let Err(err) = &claude_model_result {
                effects.push(Effect::LogError {
                    module: "claude_model".to_owned(),
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
                    ..model
                },
                effects,
            )
        }
        Msg::CacheLoaded(result) => {
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
                    Err(usage::Error::CacheRead(err.to_string()))
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
    }
}
