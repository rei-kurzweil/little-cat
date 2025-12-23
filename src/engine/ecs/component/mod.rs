pub mod renderable;
pub mod transform;

use crate::engine::ecs::entity::EntityId;
use std::any::Any;

/// Component interface.
/// `init` runs when the component is registered on an entity that is registered with the world.
pub trait Component: Any + 'static + Send + Sync {
    /// Lifecycle hook: called when the component is registered with the world.
    ///
    /// Default is intentionally a no-op so most components don't need to implement it.
    /// Override this only if the component needs to register resources/state into `World`.
    fn init(&mut self, _world: &mut crate::engine::ecs::World, _entity: EntityId) {}
}
