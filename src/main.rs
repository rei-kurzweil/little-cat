mod engine;
mod utils;

fn main() {
    utils::logger::init();

    let world = engine::ecs::World::default();

    // Demo scene is ECS-driven. RenderableSystem will convert ECS RenderableComponent
    // into VisualWorld GpuRenderable records during `Universe::update()`.
    let mut universe = engine::Universe::new(world);
    {
        use engine::ecs::component::{RenderableComponent, TransformComponent};
        use engine::ecs::entity::Entity;

        // Triangle entity
        let tri = {
            let e = universe.world.spawn_entity();
            Entity::new(e.id)
                .with_component(TransformComponent::new().with_position(-0.5, 0.0, 0.0))
                .with_component(RenderableComponent::triangle())
        };

        // Square entity
        let sq = {
            let e = universe.world.spawn_entity();
            Entity::new(e.id)
                .with_component(
                    TransformComponent::new()
                        .with_position(0.5, 0.0, 0.0)
                        .with_scale(0.75, 0.75, 1.0),
                )
                .with_component(RenderableComponent::square())
        };

        // Register entities with the universe/system world so component init hooks run.
        universe
            .world
            .add_entity(&mut universe.systems, &mut universe.visuals, tri);
        universe
            .world
            .add_entity(&mut universe.systems, &mut universe.visuals, sq);
    }
    let renderer = engine::graphics::Renderer::new();
    let user_input = engine::user_input::UserInput::new();

    engine::Windowing::run_app(universe, renderer, user_input).expect("Windowing failed");
}
