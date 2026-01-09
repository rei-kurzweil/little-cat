use crate::engine::ecs::component::{
    ColorComponent, InputComponent, PointLightComponent, RenderableComponent, TextureComponent,
    TransformComponent,
};
use crate::engine::graphics::mesh::MeshFactory;
use crate::engine::graphics::primitives::MaterialHandle;
use crate::engine::user_input::InputState;
use crate::engine::{ecs, graphics};
use std::sync::Arc;
use winit::window::Window;

pub struct Universe {
    pub world: ecs::World,
    pub command_queue: ecs::CommandQueue,
    pub systems: ecs::SystemWorld,

    pub visuals: graphics::VisualWorld,
    pub render_assets: graphics::RenderAssets,

    renderer: graphics::VulkanoRenderer,
}

impl Universe {
    pub fn new(world: ecs::World) -> Self {
        Self {
            world,
            command_queue: ecs::CommandQueue::new(),
            systems: ecs::SystemWorld::new(),

            visuals: graphics::VisualWorld::new(),
            render_assets: graphics::RenderAssets::new(),
            renderer: graphics::VulkanoRenderer::new(),
        }
    }

    /// Initialize the renderer for a window.
    /// This must be called before rendering.
    pub fn init_renderer_for_window(
        &mut self,
        window: &Arc<Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.renderer.init_for_window(window)
    }

    /// Resize the renderer when the window is resized.
    pub fn resize_renderer(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(size);
    }

    /// Build the demo scene with 7 shapes and a textured square.
    /// This can be called from main.rs after Universe creation.
    pub fn build_demo_scene_7_shapes(&mut self) {
        // Register CPU meshes once and reuse handles.
        let tri_mesh = self.render_assets.register_mesh(MeshFactory::triangle_2d());
        let square_mesh = self.render_assets.register_mesh(MeshFactory::quad_2d());

        fn spawn(
            world: &mut ecs::World,
            queue: &mut ecs::CommandQueue,
            mesh: crate::engine::graphics::primitives::CpuMeshHandle,
            x: f32,
            y: f32,
            s: f32,
            r: f32,
            color: [f32; 4],
            input_driven: bool,
        ) -> ecs::ComponentId {
            let transform = world.add_component(
                TransformComponent::new()
                    .with_position(x, y, 0.0)
                    .with_scale(s, s, 1.0)
                    .with_rotation_euler(0.0, 0.0, r),
            );
            let renderable = world.add_component(RenderableComponent::new(
                crate::engine::graphics::primitives::Renderable::new(
                    mesh,
                    MaterialHandle::TOON_MESH,
                ),
            ));
            let color_c = world.add_component(ColorComponent { rgba: color });

            // Topology: (optional Input) -> Transform -> Renderable
            let _ = world.add_child(transform, renderable);
            let _ = world.add_child(renderable, color_c);

            if input_driven {
                let input = world.add_component(InputComponent::new().with_speed(0.5));
                let _ = world.add_child(input, transform);
                world.init_component_tree(input, queue);
            } else {
                world.init_component_tree(transform, queue);
            }

            transform
        }

        // Spawn shapes.
        // One triangle is input-driven (WASD/QE). The point light is attached under the same
        // transform so it moves with the triangle.
        let tri_root_transform = self
            .world
            .add_component(TransformComponent::new().with_position(0.5, 0.50, 0.0));

        // Visual transform under the root; this is where we apply rotation/scale.
        // Rotating by PI should visually flip the triangle while leaving its input-driven
        // movement (on the root transform) unchanged.
        let tri_visual_transform = self.world.add_component(
            TransformComponent::new()
                .with_scale(0.30, 0.30, 1.0)
                .with_rotation_euler(0.0, 0.0, (2.0 * 3.14159 / 3.0) + 3.14159),
        );
        let tri_renderable = self.world.add_component(RenderableComponent::new(
            crate::engine::graphics::primitives::Renderable::new(
                tri_mesh,
                MaterialHandle::TOON_MESH,
            ),
        ));
        let tri_color = self
            .world
            .add_component(ColorComponent::rgba(0.2, 1.0, 0.2, 1.0));
        let tri_light = self.world.add_component(
            PointLightComponent::new()
                .with_distance(10.0)
                .with_color(1.0, 0.0, 0.0),
        );

        let _ = self
            .world
            .add_child(tri_root_transform, tri_visual_transform);
        let _ = self.world.add_child(tri_visual_transform, tri_renderable);
        let _ = self.world.add_child(tri_renderable, tri_color);
        let _ = self.world.add_child(tri_root_transform, tri_light);

        let tri_input = self
            .world
            .add_component(InputComponent::new().with_speed(0.5));
        let _ = self.world.add_child(tri_input, tri_root_transform);
        self.world
            .init_component_tree(tri_input, &mut self.command_queue);

        spawn(
            &mut self.world,
            &mut self.command_queue,
            square_mesh,
            -0.80,
            -0.30,
            0.25,
            0.0,
            [1.0, 0.2, 0.2, 1.0],
            false,
        );
        spawn(
            &mut self.world,
            &mut self.command_queue,
            square_mesh,
            -0.40,
            -0.30,
            0.25,
            0.0,
            [1.0, 0.6, 0.2, 1.0],
            false,
        );
        spawn(
            &mut self.world,
            &mut self.command_queue,
            square_mesh,
            0.00,
            -0.30,
            0.25,
            0.0,
            [1.0, 1.0, 0.2, 1.0],
            false,
        );
        spawn(
            &mut self.world,
            &mut self.command_queue,
            square_mesh,
            0.40,
            -0.30,
            0.25,
            0.0,
            [0.2, 0.6, 1.0, 1.0],
            false,
        );
        spawn(
            &mut self.world,
            &mut self.command_queue,
            square_mesh,
            0.80,
            -0.30,
            0.25,
            0.0,
            [0.8, 0.2, 1.0, 1.0],
            false,
        );
        spawn(
            &mut self.world,
            &mut self.command_queue,
            tri_mesh,
            0.30,
            0.35,
            0.30,
            -3.14159,
            [1.0, 1.0, 1.0, 1.0],
            false,
        );

        // Textured square.
        let tex_transform = self.world.add_component(
            TransformComponent::new()
                .with_position(0.0, 0.10, 0.0)
                .with_scale(0.45, 0.45, 1.0),
        );
        let tex_renderable = self.world.add_component(RenderableComponent::new(
            crate::engine::graphics::primitives::Renderable::new(
                square_mesh,
                MaterialHandle::TOON_MESH,
            ),
        ));
        let tex_color = self
            .world
            .add_component(ColorComponent::rgba(1.0, 1.0, 1.0, 1.0));
        let tex = self
            .world
            .add_component(TextureComponent::from_png("assets/cat-face-neutral.png"));

        let _ = self.world.add_child(tex_transform, tex_renderable);
        let _ = self.world.add_child(tex_renderable, tex_color);
        let _ = self.world.add_child(tex_renderable, tex);
        self.world
            .init_component_tree(tex_transform, &mut self.command_queue);

        // NOTE: This demo intentionally does not spawn a camera.
        // VisualWorld defaults to an identity 2D camera transform.
    }

    /// Game/update step
    pub fn update(&mut self, dt_sec: f32, input: &InputState) {
        // 1. Process input events (handled inside systems for now).
        // 2. Let systems call methods on components,
        //      for example, to update transforms or renderables, which
        //      will update VisualWorld can update draw_batches and give Renderer a snapshot
        self.systems.tick(
            &mut self.world,
            &mut self.visuals,
            input,
            &mut self.command_queue,
            dt_sec,
        );

        // Process commands after tick so any commands queued during tick are processed in the same frame
        self.systems
            .process_commands(&mut self.world, &mut self.visuals, &mut self.command_queue);
    }

    pub fn render(&mut self) {
        // Prepare render (mesh uploads) - cast renderer to trait
        self.systems.prepare_render(
            &mut self.world,
            &mut self.visuals,
            &mut self.render_assets,
            &mut self.renderer as &mut dyn graphics::RenderUploader,
        );

        // TODO: rebuild inspector around component graph instead of entities.

        self.renderer
            .render_visual_world(&mut self.visuals)
            .expect("render failed");
    }
}
