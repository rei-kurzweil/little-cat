mod engine;
mod utils;

fn main() {
    utils::logger::init();

    let world = engine::ecs::World::default();

    // Demo scene is ECS-driven. RenderableSystem will convert ECS RenderableComponent
    // into VisualWorld GpuRenderable records during `Universe::update()`.
    let mut universe = engine::Universe::new(world);
    let mut renderer = engine::graphics::Renderer::new();
    {
        use engine::ecs::component::{RenderableComponent, TransformComponent};
        use engine::ecs::entity::Entity;
        use engine::graphics::MeshFactory;

        // Register reusable CPU meshes once and share handles across components.
        let tri_mesh = universe.render_assets.register_mesh(MeshFactory::triangle_2d());
        let quad_mesh = universe.render_assets.register_mesh(MeshFactory::quad_2d());

        // Triangle entity
        let tri = {
            let e = universe.world.spawn_entity();
            Entity::new(e.id)
                .with_component(
                    TransformComponent::new().with_position(-0.5, 0.0, 0.0))
                .with_component(
                    RenderableComponent::triangle(tri_mesh))
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
                .with_component(
                    RenderableComponent::square(quad_mesh)
                )
        };

        // Register entities with the universe/system world so component init hooks run.
            // Register entities through Universe so renderable init can upload meshes.
        universe.add_entity(tri);
        universe.add_entity(sq);
    }
    let user_input = engine::user_input::UserInput::new();

    engine::Windowing::run_app(universe, renderer, user_input).expect("Windowing failed");
}
