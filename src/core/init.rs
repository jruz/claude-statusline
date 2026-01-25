use crate::core::effect;
use crate::core::state;

pub fn init() -> (state::Model, Vec<effect::Effect>) {
    (state::Model::default(), vec![])
}
