pub mod component;
pub mod entity;
pub mod system;

use std::collections::HashMap;

use crate::engine::ecs::entity::{Entity, EntityId};
use crate::engine::graphics::{RenderAssets, VisualWorld};

// Re-export these so other modules can use `crate::engine::ecs::Transform`
// and `crate::engine::ecs::Renderable` consistently.
pub use crate::engine::graphics::primitives::{Renderable, Transform};

pub use system::{CursorSystem, System, SystemWorld};

/// Bundle of mutable engine state passed to component mutation APIs.
///
/// This exists to avoid threading `&mut World`, `&mut SystemWorld`, and `&mut VisualWorld`
/// through every component call.
pub struct WorldContext<'a> {
    pub world: &'a mut World,
    pub systems: &'a mut SystemWorld,
    pub visuals: &'a mut VisualWorld,
    pub render_assets: &'a mut RenderAssets,
}

impl<'a> WorldContext<'a> {
    pub fn new(
        world: &'a mut World,
        systems: &'a mut SystemWorld,
        visuals: &'a mut VisualWorld,
        render_assets: &'a mut RenderAssets,
    ) -> Self {
        Self {
            world,
            systems,
            visuals,
            render_assets,
        }
    }
}

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

    // NOTE: We intentionally do NOT provide a renderer-dependent add_entity anymore.
    // Renderer work gets deferred to the per-frame render preparation step.

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

    /// Ensure `next_id` is at least `id + 1`.
    ///
    /// Useful when inserting externally-constructed entities with explicit ids.
    pub fn reserve_entity_id(&mut self, id: EntityId) {
        self.next_id = self.next_id.max(id.saturating_add(1));
    }

    /// Insert an entity into storage without running init hooks.
    ///
    /// This is a low-level API. Prefer `Universe::add_entity` for normal gameplay usage.
    pub fn insert_entity_raw(&mut self, e: Entity) -> EntityId {
        let id = e.id;
        self.entities.insert(id, e);
        id
    }
}
