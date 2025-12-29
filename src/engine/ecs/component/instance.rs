use super::Component;

use crate::engine::ecs::ComponentId;
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;
use crate::engine::graphics::primitives::InstanceHandle;

/// Component that holds a handle to a graphics instance.
#[derive(Debug, Clone, Copy)]
pub struct InstanceComponent {
    pub handle: Option<InstanceHandle>,
}

impl InstanceComponent {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn with_handle(mut self, handle: InstanceHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Get the instance handle. Returns None if not yet initialized.
    pub fn get_handle(&self) -> Option<InstanceHandle> {
        self.handle
    }
}

impl Component for InstanceComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(
        &mut self,
        _world: &mut World,
        _systems: &mut SystemWorld,
        _visuals: &mut crate::engine::graphics::VisualWorld,
        _component: ComponentId,
    ) {
        // Initialization logic can be added here if needed
        // For now, InstanceComponent doesn't auto-register with VisualWorld
        // Systems like RenderableSystem will handle registration
    }
}

impl Default for InstanceComponent {
    fn default() -> Self {
        Self::new()
    }
}
