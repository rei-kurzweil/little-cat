pub mod renderable;
pub mod transform;
pub mod cursor;
pub mod instance;

pub use renderable::RenderableComponent;
pub use transform::TransformComponent;
pub use cursor::CursorComponent;
pub use instance::InstanceComponent;

use crate::engine::ecs::entity::EntityId;
use crate::engine::ecs::entity::ComponentId;
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;

/// Component interface.
/// `init` runs when the component is registered on an entity that is registered with the world.
pub trait Component: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Concrete type name (for debugging / inspection).
    fn type_name(&self) -> &'static str {
        // Fast path for our known component set.
        if self.as_any().is::<crate::engine::ecs::component::InstanceComponent>() {
            return core::any::type_name::<crate::engine::ecs::component::InstanceComponent>();
        }
        if self.as_any().is::<crate::engine::ecs::component::TransformComponent>() {
            return core::any::type_name::<crate::engine::ecs::component::TransformComponent>();
        }
        if self.as_any().is::<crate::engine::ecs::component::RenderableComponent>() {
            return core::any::type_name::<crate::engine::ecs::component::RenderableComponent>();
        }
        if self.as_any().is::<crate::engine::ecs::component::CursorComponent>() {
            return core::any::type_name::<crate::engine::ecs::component::CursorComponent>();
        }

        "<unknown component>"
    }

    /// Called immediately after the component is assigned an `(EntityId, ComponentId)`.
    ///
    /// Components can override this to store identity internally so their mutation APIs
    /// don't need to take `entity`/`component` parameters.
    fn set_ids(&mut self, _entity: EntityId, _component: ComponentId) {
    }

    /// Called when component is added to an entity in the world.
    fn init(
        &mut self,
        _world: &mut World,
        _systems: &mut SystemWorld,
        _visuals: &mut crate::engine::graphics::VisualWorld,
        _entity: EntityId,
        _component: ComponentId,
    ) {
    }

    /// Called when component is removed from an entity.
    fn cleanup(
        &mut self,
        _world: &mut World,
        _systems: &mut SystemWorld,
        _visuals: &mut crate::engine::graphics::VisualWorld,
        _entity: EntityId,
        _component: ComponentId,
    ) {
    }
}
