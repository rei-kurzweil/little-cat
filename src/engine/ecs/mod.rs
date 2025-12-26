pub mod component;
pub mod entity;
pub mod system;

use std::collections::HashMap;

use crate::engine::ecs::entity::{Entity, EntityId};
use crate::engine::ecs::system::SystemWorld as EcsSystemWorld;

// Re-export these so other modules can use `crate::engine::ecs::Transform`
// and `crate::engine::ecs::Renderable` consistently.
pub use crate::engine::graphics::primitives::{Renderable, Transform};

pub use system::{CursorSystem, System, SystemWorld};

/// Extremely small world placeholder (Entity store).
#[derive(Default)]
pub struct World {
    next_id: EntityId,
    entities: HashMap<EntityId, Entity>,
}

impl World {
    /// Allocate a new entity id.
    pub fn spawn(&mut self) -> EntityId {
        self.spawn_entity().id
    }

    /// Register an entity with the world and run `init()` for all its components.
    /// Replaces any existing entity with the same id.
    pub fn add_entity(&mut self, systems: &mut EcsSystemWorld, mut e: Entity) {
        let id = e.id;
        self.next_id = self.next_id.max(id.saturating_add(1));

        e.init_all(self, systems);

        self.entities.insert(id, e);
    }

    pub fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    pub fn get_all_entities(&self) -> Vec<&Entity> {
        self.entities.values().collect()
    }

    /// Spawn a new entity id and return an `Entity` with that id (not yet registered).
    pub fn spawn_entity(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        Entity::new(id)
    }

    /// Convenience: register an entity builder immediately.
    /// This is just `add_entity`, but reads nicely at call sites that "spawn then add".
    pub fn spawn_and_add(&mut self, e: Entity) -> EntityId {
        let id = e.id;
        // If you want system registration during init, call `Universe::spawn_and_add` instead.
        // This method exists for low-level storage only.
        self.entities.insert(id, e);
        id
    }
}
