use crate::core::state;

pub fn render(model: &state::Model) -> String {
    let context = model.context.map_or("...", |()| "...");
    let claude_model = model.claude.map_or("...", |()| "...");
    let five_hour = model.usage.map_or("...", |()| "...");
    let plan = model.plan.map_or("...", |()| "...");
    let seven_day = model.usage.map_or("...", |()| "...");

    let line1 = format!("Context: {context:<14}Model: {claude_model:<11}5h: {five_hour}");
    let line2 = format!("Plan: {plan:<33}7d: {seven_day}");
    format!("{line1}\n{line2}")
}
