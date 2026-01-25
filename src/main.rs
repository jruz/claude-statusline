mod components;
mod effect;
mod model;
mod shared;
mod view;

use effect::execute;
use model::init;
use model::update;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let (mut model, mut effects) = init();
    while let Some(effect) = effects.pop() {
        if let Some(msg) = execute(&effect) {
            let (new_model, new_effects) = update(model, msg);
            model = new_model;
            effects.extend(new_effects);
        }
    }

    println!("{}", view::render(&model));
    Ok(())
}
