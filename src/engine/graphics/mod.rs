pub mod primitives;
pub mod renderer;
pub mod visual_world;

pub use renderer::{MaterialHandle, MeshHandle, Renderer};
pub use visual_world::{Instance, VisualWorld};

/// Graphics/Vulkan placeholder.
pub struct Graphics;

impl Graphics {
    pub fn new() -> Self {
        Self
    }
}

