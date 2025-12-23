use crate::engine::{ecs, graphics};

pub struct Universe {
    pub world: ecs::World,
    pub visual_world: graphics::visual_world::VisualWorld,
}

impl Universe {
    pub fn new(world: ecs::World) -> Self {
        Self {
            world,
            visual_world: graphics::visual_world::VisualWorld::new(),
        }
    }

    /// Pull render-relevant state from ECS and update the renderer-facing cache.
    /// For now this can rebuild each frame; later switch to events/dirty tracking.
    pub fn sync_visuals(&mut self) {
        self.visual_world.clear();
        self.visual_world.extend_from_iter(self.world.query_renderables());
    }

    pub fn render(&mut self, renderer: &mut graphics::Renderer) {
        renderer.render_visual_world(&self.visual_world);
    }
}