// Public renderer-owned resource handles.
// NOTE: These are defined here for now; you may later move/re-export them from `engine::graphics::mod`.


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialHandle(pub u32);

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    // Render a frame given some view of world/scene state.
    pub fn render_visual_world(&mut self, visual_world: &crate::engine::graphics::visual_world::VisualWorld) {
        for (_renderable, _instances) in visual_world.groups() {
            // TODO:
            // - bind pipeline/material for `_renderable`
            // - bind mesh buffers for `_renderable.mesh`
            // - upload instance buffer from `_instances`
            // - issue instanced draw
        }
    }

    // Keep or remove this wrapper as you prefer; leaving it is fine for now.
    pub fn render(
        &mut self,
        renderables: impl IntoIterator<
            Item = (
                crate::engine::ecs::entity::EntityId,
                crate::engine::ecs::Transform,
                crate::engine::ecs::Renderable,
            ),
        >,
    ) {
        let mut vw = crate::engine::graphics::visual_world::VisualWorld::new();
        vw.extend_from_iter(renderables);
        self.render_visual_world(&vw);
    }
}