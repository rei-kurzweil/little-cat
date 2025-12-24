mod engine;
mod utils;

fn main() {
    utils::logger::init();

    let world = engine::ecs::World::default();
    let universe = engine::Universe::new(world);
    let renderer = engine::graphics::Renderer::new();

    engine::Windowing::run_app(universe, renderer).expect("Windowing failed");
}
