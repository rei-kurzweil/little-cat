use crate::engine::{ecs, graphics};

pub struct Universe {
    pub world: ecs::World,
    pub visuals: graphics::VisualWorld,
}

impl Universe {
    pub fn new(world: ecs::World) -> Self {
        Self {
            world,
            visuals: graphics::VisualWorld::new(),
        }
    }

    /// Game/update step (placeholder).
    pub fn update(&mut self, _dt_sec: f32) {
        // TODO: run systems, gameplay, physics, etc.
    }

    /// Bridge ECS -> renderer-friendly cache.
    /// For now: rebuild each frame (simple, correct). Later: events/dirty tracking.
    pub fn sync_visuals(&mut self) {
        self.visuals.clear();
        self.visuals.extend_from_iter(self.world.query_renderables());
    }

    pub fn render(&mut self, renderer: &mut graphics::Renderer) {
        renderer.render_visual_world(&self.visuals);
    }
}