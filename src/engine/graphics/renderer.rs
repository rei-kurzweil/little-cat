// Public renderer-owned resource handles.
// NOTE: Handle types live in `graphics/primitives.rs` now.

use crate::engine::{EngineError, EngineResult};
use crate::engine::graphics::{Material, MaterialHandle, VisualWorld};
use crate::engine::graphics::primitives::{Mesh, MeshHandle};
use winit::dpi::PhysicalSize;
use winit::window::Window;


pub struct Renderer {
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
    /// Temporary: draw even when there is no scene data, to validate window/swapchain.
    pub debug_draw_hardcoded_triangle: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            // Put built-ins in a stable order so handles are predictable.
            materials: vec![Material::UNLIT_FULLSCREEN],
            meshes: Vec::new(), // TODO: push built-in cube/tetra meshes once buffers exist
            debug_draw_hardcoded_triangle: true,
        }
    }

    pub fn material(&self, h: MaterialHandle) -> Option<&Material> {
        self.materials.get(h.0 as usize)
    }

    pub fn mesh(&self, h: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(h.0 as usize)
    }

    // Render a frame given some view of world/scene state.
    pub fn render_visual_world(&mut self, visual_world: &VisualWorld) {
        let mut drew_any = false;

        for (renderable, _instances) in visual_world.groups() {
            let _mat = self.material(renderable.material);
            let _mesh = self.mesh(renderable.mesh);

            // TODO: only draw if both exist
            // TODO:
            // - compile/load _mat.vertex_shader + _mat.fragment_shader into shader modules
            // - create/bind pipeline

            // TODO: issue draw (for now you can hardcode vkCmdDraw(3,1,0,0) once Vulkan is wired)

            // If we reach here, we *intend* to draw something eventually.
            drew_any = true;
        }

        if !drew_any && self.debug_draw_hardcoded_triangle {
            // TODO (Vulkan wiring):
            // - bind the built-in UNLIT_FULLSCREEN material pipeline
            // - issue fullscreen triangle draw:
            //   vkCmdDraw(cmd, 3, 1, 0, 0);
            //
            // For now this is a stub hook so the control flow is correct even with an empty scene.
            // You can put logging here if you want to confirm it's executed.
        }
    }

    pub fn init_for_window(&mut self, _window: &Window) -> EngineResult<()> {
        // TODO(Vulkan): create ash Entry/Instance, surface, pick physical device,
        // create logical device/queues, create swapchain + image views, etc.
        Ok(())
    }

    pub fn resize(&mut self, _new_size: PhysicalSize<u32>) {
        // TODO(Vulkan): recreate swapchain and dependent resources
    }

    pub fn draw_frame(&mut self, visual_world: &VisualWorld) -> EngineResult<()> {
        // For now just exercise the scene traversal; later this will acquire/present.
        self.render_visual_world(visual_world);
        Ok(())
    }
}

// NOTE: winit does not provide a Vulkan context. You create `ash::Instance` and a `VkSurfaceKHR`
// using the window's raw handles (commonly via `raw-window-handle` + `ash-window`).