use crate::components::claude_model;
use crate::components::context;
use crate::components::effort;
use crate::components::plan;
use crate::components::usage;
use crate::components::version;
use crate::model::Model;
use crate::shared::grid;

const COL1_WIDTH: usize = 35;
const COL2_WIDTH: usize = 22;
const LOADING_5H: &str = "5h: ...";
const LOADING_7D: &str = "7d: ...";

pub fn render(model: &Model) -> String {
    let context_ref = model.context.as_ref();
    let context_text = context::render(context_ref);
    let claude_ref = model.claude.as_ref();
    let model_text = claude_model::render(claude_ref);
    let plan_ref = model.plan.as_ref();
    let plan_text = plan::render(plan_ref);
    let version_ref = model.version.as_ref();
    let version_text = version::render(version_ref);
    let effort_ref = model.effort.as_ref();
    let effort_text = effort::render(effort_ref);
    let (five_hour, seven_day) = parse_usage_lines(model);

    let line1 = format!(
        "{}{}{}",
        grid::pad_right(&context_text, COL1_WIDTH),
        grid::pad_right(&model_text, COL2_WIDTH),
        five_hour
    );

    let line2 = format!(
        "{}{}{}",
        grid::pad_right(&plan_text, COL1_WIDTH),
        grid::pad_right(&effort_text, COL2_WIDTH),
        seven_day
    );

    let line3 = version_text;

    format!("{line1}\n{line2}\n{line3}\n\u{200B}")
}

fn parse_usage_lines(model: &Model) -> (String, String) {
    let usage_ref = model.usage.as_ref();
    let usage_lines = usage::render(usage_ref);
    let mut usage_iter = usage_lines.lines();
    match (usage_iter.next(), usage_iter.next()) {
        (Some(five), Some(seven)) => (five.to_string(), seven.to_string()),
        (Some(five), None) => (five.to_string(), LOADING_7D.to_string()),
        (None, Some(seven)) => (LOADING_5H.to_string(), seven.to_string()),
        (None, None) => (LOADING_5H.to_string(), LOADING_7D.to_string()),
    }
}
