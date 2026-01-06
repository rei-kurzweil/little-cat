use crate::engine::ecs::component::Component;
use crate::engine::ecs::ComponentId;
use crate::engine::graphics::mesh::MeshFactory;
use crate::engine::graphics::primitives::{InstanceHandle, MaterialHandle, Renderable};

/// Renderable component.
#[derive(Debug, Clone)]
pub struct RenderableComponent {
    pub renderable: Renderable,

    /// VisualWorld instance handle created for this renderable.
    pub handle: Option<InstanceHandle>,

    component: Option<ComponentId>,
}

impl RenderableComponent {
    pub fn new(renderable: Renderable) -> Self {
        Self {
            renderable,
            handle: None,
            component: None,
        }
    }

    fn from_cpu_mesh_handle(h: crate::engine::graphics::primitives::CpuMeshHandle, material: MaterialHandle) -> Self {
        Self::new(Renderable::new(h, material))
    }

    pub fn get_handle(&self) -> Option<InstanceHandle> {
        self.handle
    }

    /// Predefined renderable: 2D triangle (placeholder handle).
    pub fn triangle(mesh: crate::engine::graphics::primitives::CpuMeshHandle) -> Self {
        let _ = MeshFactory::triangle_2d();
        Self::from_cpu_mesh_handle(mesh, MaterialHandle::TOON_MESH)
    }

    /// Predefined renderable: 2D square/quad (placeholder handle).
    pub fn square(mesh: crate::engine::graphics::primitives::CpuMeshHandle) -> Self {
        let _ = MeshFactory::quad_2d();
        Self::from_cpu_mesh_handle(mesh, MaterialHandle::TOON_MESH)
    }

    /// Predefined renderable: cube primitive (placeholder handles for now).
    pub fn cube(mesh: crate::engine::graphics::primitives::CpuMeshHandle) -> Self {
        let _ = MeshFactory::cube();
        Self::from_cpu_mesh_handle(mesh, MaterialHandle::TOON_MESH)
    }

    /// Predefined renderable: tetrahedron primitive (placeholder handles for now).
    pub fn tetrahedron(mesh: crate::engine::graphics::primitives::CpuMeshHandle) -> Self {
        let _ = MeshFactory::tetrahedron();
        Self::from_cpu_mesh_handle(mesh, MaterialHandle::TOON_MESH)
    }

    /// Predefined renderable: tetrahedron (alias of `tetrahedron`).
    pub fn color_tetrahedron(mesh: crate::engine::graphics::primitives::CpuMeshHandle) -> Self {
        let _ = MeshFactory::tetrahedron();
        Self::from_cpu_mesh_handle(mesh, MaterialHandle::TOON_MESH)
    }
}

impl Component for RenderableComponent {
    fn name(&self) -> &'static str {
        "renderable"
    }

    fn set_id(&mut self, component: ComponentId) {
        self.component = Some(component);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(
        &mut self,
        queue: &mut crate::engine::ecs::CommandQueue,
        component: ComponentId,
    ) {
        // Queue registration command instead of immediately registering
        queue.queue_register_renderable(component);
    }
}
