pub type EntityId = u64;

/// Component lifecycle hook.
/// Called when the entity is registered with the world or when the component is added
/// to an already-registered entity.
use crate::engine::ecs::component::Component;

pub struct Entity {
    pub id: EntityId,
    // `Vec<T>` is Rust's growable array (like a List/dynamic array).
    // `dyn Component` is a trait object: allows storing different component concrete types together.
    pub components: Vec<Box<dyn Component>>,
}

impl core::fmt::Debug for Entity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Entity")
            .field("id", &self.id)
            .field("components_len", &self.components.len())
            .finish()
    }
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self { id, components: Vec::new() }
    }

    /// Add a component (init will be run by `World` when appropriate).
    pub fn with_component(mut self, c: impl Component + 'static) -> Self {
        self.components.push(Box::new(c));
        self
    }

    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        // `downcast_ref::<T>()` yields:
        // - `Some(&T)` if the underlying concrete component type is `T`
        // - `None` otherwise
        //
        // `find_map` returns the first `Some(...)` result it encounters.
        self.components
            .iter()
            .find_map(|c| (c.as_ref() as &dyn std::any::Any).downcast_ref::<T>())
    }

    pub fn has_component<T: 'static>(&self) -> bool {
        self.get_component::<T>().is_some()
    }

    // Example usage:
    // let rc: Option<&RenderableComponent> = entity.get_component::<RenderableComponent>();
    // let has_t = entity.has_component::<TransformComponent>();
}

