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

/// Trait for uploading CPU meshes to GPU.
/// This abstraction allows different renderer implementations (ash, vulkano, etc.)
/// to provide mesh uploading functionality without exposing renderer-specific details.
pub trait MeshUploader {
    fn upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>>;
}

/// Graphics/Vulkan placeholder.
pub struct Graphics;

impl Graphics {
    pub fn new() -> Self {
        Self
    }
}

