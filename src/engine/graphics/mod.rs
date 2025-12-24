pub mod primitives;
pub mod renderer;
pub mod visual_world;

pub use primitives::{Material, MaterialHandle, MeshHandle, Renderable, Transform};
pub use renderer::Renderer;
pub use visual_world::{Instance, VisualWorld};

/// Graphics/Vulkan placeholder.
pub struct Graphics;

impl Graphics {
    pub fn new() -> Self {
        Self
    }
}

