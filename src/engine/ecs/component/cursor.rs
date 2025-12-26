use super::Component;
use crate::engine::ecs::entity::{ComponentId, EntityId};
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;

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

    fn init(&mut self, world: &World, systems: &mut SystemWorld, entity: EntityId, component: ComponentId) {
        systems.register_cursor(entity, component);

        // Initialize all child components.
        // Child component ids are allocated locally here for now; later we'll integrate
        // child registration into Entity/World so ids are stable and addressable.
        let mut next_child_id: ComponentId = 0;
        for child in &mut self.children {
            let cid = next_child_id;
            next_child_id = next_child_id.wrapping_add(1);
            child.init(world, systems, entity, cid);
        }
    }
}

impl core::fmt::Debug for CursorComponent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CursorComponent")
            .field("components_len", &self.children.len())
            .finish()
    }
}