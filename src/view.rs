use crate::components::claude_model;
use crate::components::context;
use crate::components::plan;
use crate::model::Model;

pub fn render(model: &Model) -> String {
    format!(
        "{}    {}\n{}",
        context::render(model.context.as_ref()),
        claude_model::render(model.claude.as_ref()),
        plan::render(model.plan.as_ref())
    )
}
