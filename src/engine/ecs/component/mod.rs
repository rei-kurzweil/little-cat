pub mod renderable;
pub mod transform;
pub mod cursor;
pub mod instance;
pub mod camera;
pub mod camera2d;
pub mod input;

pub use renderable::RenderableComponent;
pub use transform::TransformComponent;
pub use cursor::CursorComponent;
pub use instance::InstanceComponent;
pub use camera::CameraComponent;
pub use camera2d::Camera2DComponent;
pub use input::InputComponent;


/// World-owned record for a component payload plus its topology.
///
/// This is the building block of the component-centric ECS: a single flat store of records
/// in `World`, each record carrying its own parent/children handles.

pub struct ComponentNode {
    pub component: Box<dyn Component>,
    pub parent: Option<crate::engine::ecs::ComponentId>,
    pub children: Vec<crate::engine::ecs::ComponentId>,
}

impl ComponentNode {
    pub fn new(component: Box<dyn Component>) -> Self {
        Self {
            component,
            parent: None,
            children: Vec::new(),
        }
    }
}

/// Component interface.
/// `init` runs when the component is registered 
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
        if self.as_any().is::<crate::engine::ecs::component::CameraComponent>() {
            return core::any::type_name::<crate::engine::ecs::component::CameraComponent>();
        }
        if self.as_any().is::<crate::engine::ecs::component::Camera2DComponent>() {
            return core::any::type_name::<crate::engine::ecs::component::Camera2DComponent>();
        }
        if self.as_any().is::<crate::engine::ecs::component::InputComponent>() {
            return core::any::type_name::<crate::engine::ecs::component::InputComponent>();
        }

        "<unknown component>"
    }

    fn set_id(
        &mut self,
        _component: crate::engine::ecs::ComponentId,
    ) {
    }

    /// Called when component is added to the World
    fn init(
        &mut self,
        _queue: &mut crate::engine::ecs::CommandQueue,
        _component: crate::engine::ecs::ComponentId,
    ) {
    }

    /// Called when component is removed from the World.
    fn cleanup(
        &mut self,
        _queue: &mut crate::engine::ecs::CommandQueue,
        _component: crate::engine::ecs::ComponentId,
    ) {
    }
}
