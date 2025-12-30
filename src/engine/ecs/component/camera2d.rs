use crate::engine::ecs::component::Component;
use crate::engine::ecs::ComponentId;

/// 2D camera component.
///
/// This is a sibling of `CameraComponent` (3D-ish view/proj camera).
/// The 2D camera drives a global NDC translation used by the mesh vertex shader.
#[derive(Debug, Clone, Default)]
pub struct Camera2DComponent {
	handle: Option<crate::engine::ecs::system::camera_system::CameraHandle>,
}

impl Camera2DComponent {
	pub fn new() -> Self {
		Self { handle: None }
	}
}

impl Component for Camera2DComponent {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}

	fn init(
		&mut self,
		_queue: &mut crate::engine::ecs::CommandQueue,
		_component: ComponentId,
	) {
		// TODO: Queue REGISTER_CAMERA command when implemented
		// For now, camera registration is handled elsewhere
	}
}

