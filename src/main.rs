mod engine;
mod utils;

fn main() {
    utils::logger::init();

    let world = engine::ecs::World::default();
    let universe = engine::Universe::new(world);
    let user_input = engine::user_input::UserInput::new();

    engine::Windowing::run_app(universe, user_input).expect("Windowing failed");
}
