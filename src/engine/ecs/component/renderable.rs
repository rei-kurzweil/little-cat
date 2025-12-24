use crate::engine::ecs::component::Component;
use crate::engine::graphics::primitives::{MaterialHandle, MeshHandle, Renderable};

/// Renderable component.
#[derive(Debug, Clone, Copy)]
pub struct RenderableComponent {
    pub renderable: Renderable,
}

impl RenderableComponent {
    /// Predefined renderable: cube primitive (placeholder handles for now).
    pub fn cube() -> Self {
        Self {
            renderable: Renderable::new(MeshHandle::CUBE, MaterialHandle::UNLIT_FULLSCREEN),
        }
    }

    /// Predefined renderable: tetrahedron primitive (placeholder handles for now).
    pub fn tetrahedron() -> Self {
        Self {
            renderable: Renderable::new(MeshHandle::TETRAHEDRON, MaterialHandle::UNLIT_FULLSCREEN),
        }
    }
}

impl Component for RenderableComponent {}
