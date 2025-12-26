mod engine;
mod utils;

fn main() {
    utils::logger::init();

    let world = engine::ecs::World::default();
    let universe = engine::Universe::new(world);
    let renderer = engine::graphics::Renderer::new();
    let user_input = engine::user_input::UserInput::new();

    engine::Windowing::run_app(universe, renderer, user_input).expect("Windowing failed");
}
