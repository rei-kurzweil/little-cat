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

        use engine::graphics::MeshFactory;

        // Register reusable CPU meshes once and share handles across components.
        let tri_mesh = universe.render_assets.register_mesh(MeshFactory::triangle_2d());
        let quad_mesh = universe.render_assets.register_mesh(MeshFactory::quad_2d());

        // Spawn a small grid of shapes. These are already in clip-ish units, so keep
        // the positions relatively small and scale down a bit.
        let positions: &[(f32, f32)] = &[
            (-0.75, -0.5),
            (-0.25, -0.5),
            (0.25, -0.5),
            (0.75, -0.5),
            (-0.75, 0.1),
            (-0.25, 0.1),
            (0.25, 0.1),
            (0.75, 0.1),
            (-0.5, 0.65),
            (0.0, 0.65),
            (0.5, 0.65),
        ];

        for (i, &(x, y)) in positions.iter().enumerate() {
            let is_tri = (i % 2) == 0;
            let e = universe.world.spawn_entity();

            let mut ent = Entity::new(e.id).with_component(
                TransformComponent::new()
                    // Move the whole scene slightly away from the camera so perspective is visible.
                    .with_position(x, y, -2.0)
                    .with_scale(0.25, 0.25, 1.0),
            );

            if is_tri {
                ent = ent.with_component(RenderableComponent::triangle(tri_mesh));
            } else {
                ent = ent.with_component(RenderableComponent::square(quad_mesh));
            }

            universe.add_entity(ent);
        }

        // (Camera intentionally omitted during ECS-id migration.)
    }
    let user_input = engine::user_input::UserInput::new();

    engine::Windowing::run_app(universe, renderer, user_input).expect("Windowing failed");
}
