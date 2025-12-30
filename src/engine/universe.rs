use crate::engine::{ecs, graphics};
use crate::engine::user_input::InputState;
use crate::engine::ecs::component::{InstanceComponent, RenderableComponent, TransformComponent};
use crate::engine::graphics::mesh::MeshFactory;
use crate::engine::graphics::primitives::MaterialHandle;


pub struct Universe {
    pub world: ecs::World,
    pub visuals: graphics::VisualWorld,
    pub render_assets: graphics::RenderAssets,
    pub systems: ecs::SystemWorld,
    pub command_queue: ecs::CommandQueue,

}

impl Universe {
    pub fn new(world: ecs::World) -> Self {
        let mut u = Self {
            world,
            visuals: graphics::VisualWorld::new(),
            render_assets: graphics::RenderAssets::new(),
            systems: ecs::SystemWorld::new(),
            command_queue: ecs::CommandQueue::new(),
        };

        // Temporary: rebuild a demo scene directly in Universe creation.
        // This keeps runtime visuals alive while we finalize a proper scene/level layer.
        u.build_demo_scene_7_shapes();

        u
    }

    fn build_demo_scene_7_shapes(&mut self) {
        // Register CPU meshes once and reuse handles.
        let tri_mesh = self.render_assets.register_mesh(MeshFactory::triangle_2d());
        let square_mesh = self.render_assets.register_mesh(MeshFactory::quad_2d());
        

        // Helper to spawn a single rendered shape.
        let mut spawn = |mesh, x: f32, y: f32, s: f32, r: f32| {
            

            let instance = self.world.add_component(InstanceComponent::new());
            let transform = self.world.add_component(
                TransformComponent::new()
                    .with_position(x, y, 0.0)
                    .with_scale(s, s, 1.0)
                    .with_rotation_euler(0.0, 0.0, r)
            );
            let renderable = self.world.add_component(RenderableComponent {
                renderable: crate::engine::graphics::primitives::Renderable::new(
                                mesh, MaterialHandle::UNLIT_MESH
                            ),
            });

            
            // Attach under the InstanceComponent (RenderableSystem expects this topology).
            let _ = self.world.add_child(instance, transform);
            let _ = self.world.add_child(instance, renderable);

            // Initialize the component tree starting from the instance
            // This will recursively initialize all children (transform, renderable)
            self.world.init_component_tree(instance, &mut self.command_queue);
        };

        // 5 squares
        spawn(square_mesh, -0.80, -0.30, 0.25, 0.0);
        spawn(square_mesh, -0.40, -0.30, 0.25, 0.0);
        spawn(square_mesh, 0.00, -0.30, 0.25, 0.0);
        spawn(square_mesh, 0.40, -0.30, 0.25, 0.0);
        spawn(square_mesh, 0.80, -0.30, 0.25, 0.0);

        // 2 triangles
        spawn(tri_mesh, -0.20, 0.35, 0.30, 3.14159 / 2.0);
        spawn(tri_mesh, 0.30, 0.35, 0.30, -3.14159);
    }

    /// Game/update step
    pub fn update(&mut self, dt_sec: f32, input: &InputState) {
        // Debug: check if update is being called
        use std::sync::atomic::{AtomicU32, Ordering};
        static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
        let frame = FRAME_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if frame % 60 == 0 {
            println!("[Universe] update called (frame {}, dt={:.3})", frame, dt_sec);
        }
        
        // 1. Process input events (handled inside systems for now).
        // 2. Let systems call methods on components,
        //      for example, to update transforms or renderables, which
        //      will update VisualWorld can update draw_batches and give Renderer a snapshot
        self.systems.tick(&mut self.world, &mut self.visuals, input, &mut self.command_queue, dt_sec);
        
        // Process commands after tick so any commands queued during tick are processed in the same frame
        self.systems.process_commands(&mut self.world, &mut self.visuals, &mut self.command_queue);
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