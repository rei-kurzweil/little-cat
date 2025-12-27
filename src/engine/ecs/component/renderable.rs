use crate::engine::ecs::component::Component;
use crate::engine::ecs::entity::{ComponentId, EntityId};
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;
use crate::engine::graphics::primitives::{MaterialHandle, MeshHandle, Renderable};

/// Renderable component.
#[derive(Debug, Clone, Copy)]
pub struct RenderableComponent {
    pub renderable: Renderable,
}

impl RenderableComponent {
    /// Predefined renderable: 2D triangle (placeholder handle).
    pub fn triangle() -> Self {
        Self {
            renderable: Renderable::new(MeshHandle::TRIANGLE, MaterialHandle::UNLIT_FULLSCREEN),
        }
    }

    /// Predefined renderable: 2D square/quad (placeholder handle).
    pub fn square() -> Self {
        Self {
            renderable: Renderable::new(MeshHandle::SQUARE, MaterialHandle::UNLIT_FULLSCREEN),
        }
    }

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

    /// Predefined renderable: tetrahedron with a screen-space XY gradient material.
    pub fn color_tetrahedron() -> Self {
        Self {
            renderable: Renderable::new(MeshHandle::TETRAHEDRON, MaterialHandle::GRADIENT_BG_XY),
        }
    }
}

impl Component for RenderableComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(
        &mut self,
        world: &mut World,
        systems: &mut SystemWorld,
        visuals: &mut crate::engine::graphics::VisualWorld,
        entity: EntityId,
        component: ComponentId,
    ) {
        systems.register_renderable(world, visuals, entity, component);
    }
}
