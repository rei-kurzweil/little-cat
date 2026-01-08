use crate::engine::ecs::ComponentId;
use crate::engine::ecs::component::Component;

/// 2D camera component.
///
/// This is a sibling of `Camera3DComponent` (3D-ish view/proj camera).
/// The 2D camera drives a global NDC translation used by the mesh vertex shader.
#[derive(Debug, Clone, Default)]
pub struct Camera2DComponent {
    pub handle: Option<crate::engine::ecs::system::camera_system::CameraHandle>,
}

impl Camera2DComponent {
    pub fn new() -> Self {
        Self { handle: None }
    }
}

impl Component for Camera2DComponent {
    fn name(&self) -> &'static str {
        "camera2d"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self, queue: &mut crate::engine::ecs::CommandQueue, component: ComponentId) {
        queue.queue_register_camera2d(component);
    }

    fn encode(&self) -> std::collections::HashMap<String, serde_json::Value> {
        // Camera2D has no persistent state beyond the handle (which is runtime-only)
        std::collections::HashMap::new()
    }

    fn decode(
        &mut self,
        _data: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        // Handle will be regenerated during init()
        Ok(())
    }
}
