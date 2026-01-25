use crate::core::effect;
use crate::core::msg;
use crate::core::state;

pub fn update(model: state::Model, _msg: msg::Msg) -> (state::Model, Vec<effect::Effect>) {
    (model, vec![])
}
