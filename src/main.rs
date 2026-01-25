mod components;
mod core;

use crate::core::init;
use crate::core::runtime;
use crate::core::update;
use crate::core::view;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let (mut model, init_effects) = init::init();
    let messages = runtime::execute_effects(init_effects);

    for msg in messages {
        let (new_model, _effects) = update::update(model, msg);
        model = new_model;
    }

    let output = view::render(&model);
    println!("{output}");

    Ok(())
}
