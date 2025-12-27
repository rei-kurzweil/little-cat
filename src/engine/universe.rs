use crate::engine::{ecs, graphics};
use crate::engine::user_input::InputState;

pub struct Universe {
    pub world: ecs::World,
    pub visuals: graphics::VisualWorld,
    pub systems: ecs::SystemWorld,
}

impl Universe {
    pub fn new(world: ecs::World) -> Self {
        Self {
            world,
            visuals: graphics::VisualWorld::new(),
            systems: ecs::SystemWorld::new(),
        }
    }

    /// Game/update step (placeholder).
    pub fn update(&mut self, _dt_sec: f32, _input: &InputState) {
        // 1) Refresh visuals from ECS state.
        // 2) Let systems apply per-frame visual overrides (cursor-follow, etc.).
        //
        // Later we'll move to event/dirty-driven sync and/or ECS-owned render buffers.
        // TODO: sync_visuals should be replaced by RenderableSystem/InstanceSystem
        // self.sync_visuals();
        self.systems.tick(&mut self.world, &mut self.visuals, _input);
    }

    /// Bridge ECS -> renderer-friendly cache.
    /// For now: rebuild each frame (simple, correct). Later: events/dirty tracking.
    /// TODO: This should be replaced by RenderableSystem/InstanceSystem that populate
    /// the VisualWorld during component init().
    pub fn sync_visuals(&mut self) {
        // self.visuals.clear();
        // Replaced by systems-based approach
    }

    pub fn render(&mut self, renderer: &mut graphics::Renderer) {
        renderer.render_visual_world(&mut self.visuals);
    }
}