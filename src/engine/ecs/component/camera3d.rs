use crate::engine::ecs::ComponentId;
use crate::engine::ecs::component::Component;

/// 3D camera component.
///
/// Contract:
/// - On init, registers a camera with `CameraSystem`.
/// - The most recently registered camera becomes active.
/// - Call `make_active_camera()` to explicitly set this camera active.
#[derive(Debug, Clone)]
pub struct Camera3DComponent {
    // Handle owned by CameraSystem. Filled in during init.
    pub handle: Option<crate::engine::ecs::system::camera_system::CameraHandle>,
}

impl Camera3DComponent {
    pub fn new() -> Self {
        Self { handle: None }
    }

    /// Ask the CameraSystem to make this the active camera.
    pub fn make_active_camera(
        &mut self,
        queue: &mut crate::engine::ecs::CommandQueue,
        component: ComponentId,
    ) {
        if self.handle.is_some() {
            queue.queue_make_active_camera(component);
        }
    }
}

impl Default for Camera3DComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Camera3DComponent {
    fn name(&self) -> &'static str {
        "camera3d"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self, queue: &mut crate::engine::ecs::CommandQueue, component: ComponentId) {
        queue.queue_register_camera_3d(component);
    }
}
