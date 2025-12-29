mod engine;
mod utils;

fn main() {
    utils::logger::init();

    let world = engine::ecs::World::default();

    // Demo scene is ECS-driven. RenderableSystem will convert ECS RenderableComponent
    // into VisualWorld GpuRenderable records during `Universe::update()`.
    let universe = engine::Universe::new(world);
    let renderer = engine::graphics::Renderer::new();
    // NOTE: demo scene construction is temporarily disabled while the ECS migration
    // removes Entity and moves to a World-owned component graph.
    let user_input = engine::user_input::UserInput::new();

    engine::Windowing::run_app(universe, renderer, user_input).expect("Windowing failed");
}
