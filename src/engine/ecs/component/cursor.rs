use super::Component;
use crate::engine::ecs::ComponentId;

/// Marker component that indicates this entity should follow the cursor.
pub struct CursorComponent {
    children: Vec<Box<dyn Component>>,
}

impl CursorComponent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
    
    pub fn with_component(mut self, component: Box<dyn Component>) -> Self {
        self.children.push(component);
        self
    }
}

impl Component for CursorComponent {
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
        // TODO: Queue REGISTER_CURSOR command when implemented
        // For now, cursor registration is handled elsewhere
    }
}

impl core::fmt::Debug for CursorComponent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CursorComponent")
            .field("components_len", &self.children.len())
            .finish()
    }
}