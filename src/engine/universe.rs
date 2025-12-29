use crate::engine::{ecs, graphics};
use crate::engine::user_input::InputState;


pub struct Universe {
    pub world: ecs::World,
    pub visuals: graphics::VisualWorld,
    pub render_assets: graphics::RenderAssets,
    pub systems: ecs::SystemWorld,

}

impl Universe {
    pub fn new(world: ecs::World) -> Self {
        Self {
            world,
            visuals: graphics::VisualWorld::new(),
            render_assets: graphics::RenderAssets::new(),
            systems: ecs::SystemWorld::new(),
        }
    }

    /// Game/update step (placeholder).
    pub fn update(&mut self, _dt_sec: f32, _input: &InputState) {
        
        // 1) Process input events (handled inside systems for now).
        // 2) Let systems apply per-frame visual overrides (update VisualWorld so next frame can update draw_batches and give Renderer a snapshot)
        
        self.systems.tick(&mut self.world, &mut self.visuals, _input);
    }

    pub fn render(&mut self, renderer: &mut graphics::Renderer) {
        // Ensure VisualWorld contains only GPU-ready instances.
        self.systems
            .prepare_render(&mut self.world, &mut self.visuals, &mut self.render_assets, renderer);

    // TODO: rebuild inspector around component graph instead of entities.

        renderer.render_visual_world(&mut self.visuals)
                .expect("render failed");
    }
}