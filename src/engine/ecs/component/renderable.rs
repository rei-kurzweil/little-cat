use crate::engine::ecs::component::Component;
use crate::engine::graphics::primitives::Renderable;
use crate::engine::graphics::renderer::{MaterialHandle, MeshHandle};

/// Renderable component.
#[derive(Debug, Clone, Copy)]
pub struct RenderableComponent {
    pub renderable: Renderable,
}

impl RenderableComponent {
    /// Predefined renderable: cube primitive (placeholder handles for now).
    pub fn cube() -> Self {
        Self {
            renderable: Renderable::new(MeshHandle(0), MaterialHandle(0)),
        }
    }

    /// Predefined renderable: tetrahedron primitive (placeholder handles for now).
    pub fn tetrahedron() -> Self {
        Self {
            renderable: Renderable::new(MeshHandle(1), MaterialHandle(0)),
        }
    }
}

impl Component for RenderableComponent {}
