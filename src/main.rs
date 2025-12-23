mod engine;
mod utils;

fn main() {
    utils::logger::init();

    let mut world = engine::ecs::World::default();

    // Demo scene setup belongs in main (or examples), not inside `World`.
    {
        use engine::ecs::component::renderable::RenderableComponent;
        use engine::ecs::component::transform::TransformComponent;
        use engine::ecs::entity::Entity;

        let cube = world
            .spawn_entity()
            .with_component(TransformComponent::default())
            .with_component(RenderableComponent::cube());
        world.add_entity(cube);

        let tetra = world
            .spawn_entity()
            .with_component(TransformComponent::default())
            .with_component(RenderableComponent::tetrahedron());
        world.add_entity(tetra);
    }

    let mut renderer = engine::graphics::Renderer::new();
    let mut loop_ = engine::AnimationLoop::new(&mut world, &mut renderer)
        .expect("failed to create AnimationLoop");
    loop_.start().expect("AnimationLoop failed");
}
