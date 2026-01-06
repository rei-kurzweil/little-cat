pub mod primitives;
pub mod mesh;
pub mod render_assets;
pub mod render_info;
pub mod pipeline_descriptor_set_layouts;
pub mod vulkano_renderer;
pub mod visual_world;

pub use primitives::{GpuRenderable, Material, MaterialHandle, MeshHandle, Renderable, TextureHandle, Transform};
pub use mesh::{CpuMesh, CpuVertex, MeshFactory};

pub use render_assets::RenderAssets;
pub use vulkano_renderer::VulkanoRenderer;
pub use visual_world::VisualWorld;

pub use render_info::RenderInfo;
/// Trait for uploading CPU meshes to GPU.
/// This abstraction allows different renderer implementations
/// to provide mesh uploading functionality without exposing renderer-specific details.
pub trait MeshUploader {
    fn upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>>;
}

/// Trait for uploading decoded textures to the GPU.
///
/// Textures are provided as RGBA8 pixels.
pub trait TextureUploader {
    fn upload_texture_rgba8(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<TextureHandle, Box<dyn std::error::Error>>;
}

/// Convenience super-trait for types that can upload both meshes and textures.
pub trait RenderUploader: MeshUploader + TextureUploader {}

impl<T> RenderUploader for T where T: MeshUploader + TextureUploader {}

/// Graphics/Vulkan placeholder.
pub struct Graphics;

impl Graphics {
    pub fn new() -> Self {
        Self
    }
}

