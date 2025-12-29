use crate::engine::ecs::component::Component;
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;

/// Camera component.
///
/// Contract:
/// - On init, registers a camera with `CameraSystem`.
/// - The most recently registered camera becomes active.
/// - Call `make_active_camera()` to explicitly set this camera active.
#[derive(Debug, Clone)]
pub struct CameraComponent {
    // Handle owned by CameraSystem. Filled in during init.
    handle: Option<crate::engine::ecs::system::camera_system::CameraHandle>,
}

impl CameraComponent {
    pub fn new() -> Self {
        Self { handle: None }
    }

    /// Ask the CameraSystem to make this the active camera.
    pub fn make_active_camera(
        &self,
        systems: &mut SystemWorld,
        visuals: &mut VisualWorld,
    ) {
        if let Some(h) = self.handle {
            systems.camera.set_active_camera(visuals, h);
        }
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for CameraComponent {
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
        visuals: &mut VisualWorld,
        cid: ComponentId,
    ) {
        // New registration becomes the active camera by default.
        let h = systems.camera.register_camera(world, visuals, cid);
        self.handle = Some(h);
    }
}
