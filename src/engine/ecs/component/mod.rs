pub mod renderable;
pub mod transform;

use crate::engine::ecs::entity::EntityId;
use std::any::Any;

/// Component interface.
/// `init` runs when the component is registered on an entity that is registered with the world.
pub trait Component: Any + 'static + Send + Sync {
    /// Lifecycle hook: called when the component is registered with the world.
    ///
    /// Use this only for reading world state / other components. No world mutation.
    fn init(&mut self, _world: &crate::engine::ecs::World, _entity: EntityId) {}
}
