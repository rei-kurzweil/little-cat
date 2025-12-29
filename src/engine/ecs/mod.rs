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
}
