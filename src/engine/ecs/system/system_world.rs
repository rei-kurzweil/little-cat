use super::World;
use crate::engine::ecs::entity::{ComponentId, EntityId};
use crate::engine::ecs::system::CursorSystem;
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;

/// System world that holds and runs all registered systems.
#[derive(Debug, Default)]
pub struct SystemWorld {
    pub cursor: CursorSystem,
}

impl SystemWorld {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a CursorComponent instance with the CursorSystem.
    pub fn register_cursor(&mut self, entity: EntityId, component: ComponentId) {
        self.cursor.register_cursor(entity, component);
    }
    
    pub fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState) {
        self.cursor.tick(world, visuals, input);
    }
}
