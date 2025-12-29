pub mod primitives;
pub mod mesh;
pub mod render_assets;
pub mod renderer;
pub mod spirv_reflect;
pub mod visual_world;

pub use primitives::{GpuRenderable, Material, MaterialHandle, MeshHandle, Renderable, Transform};
pub use mesh::{CpuMesh, CpuVertex, MeshFactory};
pub use render_assets::RenderAssets;
pub use renderer::Renderer;
pub use visual_world::{Instance, VisualWorld};

/// Graphics/Vulkan placeholder.
pub struct Graphics;

impl Graphics {
    pub fn new() -> Self {
        Self
    }
}

