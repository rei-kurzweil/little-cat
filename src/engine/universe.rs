use crate::engine::ecs::ComponentId;
use crate::engine::{ecs, graphics};
use crate::engine::user_input::InputState;

use crate::engine::rendering_inspector::RenderingInspector;

pub struct Universe {
    pub world: ecs::World,
    pub visuals: graphics::VisualWorld,
    pub render_assets: graphics::RenderAssets,
    pub systems: ecs::SystemWorld,

    pub inspector: RenderingInspector,
}

impl Universe {
    pub fn new(world: ecs::World) -> Self {
        Self {
            world,
            visuals: graphics::VisualWorld::new(),
            render_assets: graphics::RenderAssets::new(),
            systems: ecs::SystemWorld::new(),
            inspector: RenderingInspector::new(),
        }
    }

    /// Game/update step (placeholder).
    pub fn update(&mut self, _dt_sec: f32, _input: &InputState) {
        
        // 1) Process input events (handled inside systems for now).
        // 2) Let systems apply per-frame visual overrides (update VisualWorld so next frame can update draw_batches and give Renderer a snapshot)
        
        self.systems.tick(&mut self.world, &mut self.visuals, _input);
    }

    /// Register an entity with the Universe and run component init hooks.
    ///
    /// Renderer work is deferred to `Universe::render` (prepare_render + draw), so this
    /// does not require a renderer reference.
    pub fn add_entity(&mut self, e: ecs::entity::Entity) {
        let id = e.id;

        // Keep `next_id` monotonic even if callers provide their own ids.
        self.world.reserve_entity_id(id);

        // Run init hooks first; renderable registration doesn't require the entity to already
        // be inserted into World.
        let mut ent = e;
        ent.init_all(&mut self.world, &mut self.systems, &mut self.visuals);
        self.world.insert_entity_raw(ent);
    }

    pub fn render(&mut self, renderer: &mut graphics::Renderer) {
        // Ensure VisualWorld contains only GPU-ready instances.
        self.systems
            .prepare_render(&mut self.world, &mut self.visuals, &mut self.render_assets, renderer);

        // ECS + component tree dump (prints only when summary changes unless configured otherwise).
        // For now, we don't pass instance buffer stats here; renderer prints those later.
        self.inspector.print_entity_tree(&self.world);

        renderer.render_visual_world(&mut self.visuals)
                .expect("render failed");
    }
}