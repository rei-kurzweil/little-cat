pub mod component;
pub mod system;

use slotmap::{new_key_type, SlotMap};
use crate::engine::graphics::{RenderAssets, VisualWorld};

new_key_type! {
    /// Global component identity (dense arena key).
    pub struct ComponentId;
}

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

/// World: owns all global components.
#[derive(Default)]
pub struct World {
    components: SlotMap<ComponentId, crate::engine::ecs::component::ComponentNode>,
}

impl World {
    /// Add a new component to the world (no parent). Returns its global id.
    pub fn add_component_boxed(
        &mut self,
        c: Box<dyn crate::engine::ecs::component::Component>,
    ) -> ComponentId {
        self.components
            .insert(crate::engine::ecs::component::ComponentNode::new(c))
    }

    /// Temporary alias during migration.
    pub fn spawn_component_boxed(
        &mut self,
        c: Box<dyn crate::engine::ecs::component::Component>,
    ) -> ComponentId {
        self.add_component_boxed(c)
    }

    pub fn get_component_record(&self, id: ComponentId) -> Option<&crate::engine::ecs::component::ComponentNode> {
        self.components.get(id)
    }

    pub fn get_component_record_mut(&mut self, id: ComponentId) -> Option<&mut crate::engine::ecs::component::ComponentNode> {
        self.components.get_mut(id)
    }

    // --- Topology helpers (component-graph) ---
    pub fn parent_of(&self, c: ComponentId) -> Option<ComponentId> {
        self.get_component_record(c)?.parent
    }

    pub fn children_of(&self, c: ComponentId) -> &[ComponentId] {
        static EMPTY: [ComponentId; 0] = [];
        self.get_component_record(c)
            .map(|n| n.children.as_slice())
            .unwrap_or(&EMPTY)
    }

    // --- Typed component access ---
    pub fn get_component_by_id_as<T: 'static>(&self, c: ComponentId) -> Option<&T> {
        let node = self.get_component_record(c)?;
        node.component.as_any().downcast_ref::<T>()
    }

    pub fn get_component_by_id_as_mut<T: 'static>(&mut self, c: ComponentId) -> Option<&mut T> {
        let node = self.get_component_record_mut(c)?;
        node.component.as_any_mut().downcast_mut::<T>()
    }

    pub fn get_parent_as<T: 'static>(&self, c: ComponentId) -> Option<(ComponentId, &T)> {
        let parent = self.parent_of(c)?;
        let typed = self.get_component_by_id_as::<T>(parent)?;
        Some((parent, typed))
    }

    pub fn get_parent_as_mut<T: 'static>(&mut self, c: ComponentId) -> Option<(ComponentId, &mut T)> {
        let parent = self.parent_of(c)?;
        // Avoid borrowing self twice by doing the downcast via the node.
        let node = self.get_component_record_mut(parent)?;
        let typed = node.component.as_any_mut().downcast_mut::<T>()?;
        Some((parent, typed))
    }
}
