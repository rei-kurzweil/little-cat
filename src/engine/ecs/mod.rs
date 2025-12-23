pub mod component;
pub mod entity;
pub mod system;

use std::collections::HashMap;

use crate::engine::ecs::component::Component;
use crate::engine::ecs::component::renderable::RenderableComponent;
use crate::engine::ecs::component::transform::TransformComponent;
use crate::engine::ecs::entity::{Entity, EntityId};

// Re-export these so other modules can use `crate::engine::ecs::Transform`
// and `crate::engine::ecs::Renderable` consistently.
pub use crate::engine::graphics::primitives::{Renderable, Transform};

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
    pub fn add_entity(&mut self, mut e: Entity) {
        let id = e.id;

        // Keep ids monotonic if user supplies explicit ids.
        self.next_id = self.next_id.max(id.saturating_add(1));

        // Run component init hooks before storing.
        for c in e.components.iter_mut() {
            c.init(self, id);
        }

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

    /// Add a component to an already-registered entity (runs `init()` immediately).
    pub fn add_component(&mut self, id: EntityId, mut c: Box<dyn Component>) {
        if let Some(e) = self.entities.get_mut(&id) {
            c.init(self, id);
            e.components.push(c);
        }
    }

    /// Temporary render query placeholder.
    /// TODO: once components actually register into world storage, query from that storage.
    pub fn query_renderables(&self) -> Vec<(EntityId, Transform, Renderable)> {
        let mut out = Vec::new();

        for entity in self.entities.values() {
            let t = entity.get_component::<TransformComponent>().map(|c| c.transform);
            let r = entity
                .get_component::<RenderableComponent>()
                .map(|c| c.renderable);

            if let (Some(t), Some(r)) = (t, r) {
                out.push((entity.id, t, r));
            }
        }

        out
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
        self.add_entity(e);
        id
    }
}
