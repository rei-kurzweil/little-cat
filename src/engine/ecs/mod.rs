pub mod component;
pub mod system;
pub mod command_queue;

#[cfg(test)]
mod world_graph_tests;

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
pub use command_queue::CommandQueue;

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
    /// Add a new component to the world (no parent) and return its id.
    ///
    /// Note: this currently does *not* call `Component::init`. That should happen via a
    /// higher-level API that has access to `SystemWorld` and `VisualWorld`.
    pub fn add_component<T: crate::engine::ecs::component::Component>(&mut self, c: T) -> ComponentId {
        // We set the id after insertion so components that cache their id can do so.
        let id = self.add_component_boxed(Box::new(c));
        if let Some(node) = self.get_component_record_mut(id) {
            node.component.set_id(id);
        }
        id
    }

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

    // --- Graph mutation ---
    fn is_ancestor_of(&self, maybe_ancestor: ComponentId, mut node: ComponentId) -> bool {
        while let Some(p) = self.parent_of(node) {
            if p == maybe_ancestor {
                return true;
            }
            node = p;
        }
        false
    }

    /// Attach `child` under `parent`.
    ///
    /// Safety rules:
    /// - Both ids must exist.
    /// - `child` is detached from its current parent first.
    /// - Cycles are rejected.
    pub fn add_child(&mut self, parent: ComponentId, child: ComponentId) -> Result<(), &'static str> {
        if self.get_component_record(parent).is_none() {
            return Err("parent does not exist");
        }
        if self.get_component_record(child).is_none() {
            return Err("child does not exist");
        }
        if parent == child {
            return Err("cannot parent component to itself");
        }
        if self.is_ancestor_of(child, parent) {
            return Err("cycle detected");
        }

        self.detach_from_parent(child);

        // Set child's parent.
        {
            let child_node = self.get_component_record_mut(child).ok_or("child missing")?;
            child_node.parent = Some(parent);
        }
        // Push into parent's children list.
        {
            let parent_node = self.get_component_record_mut(parent).ok_or("parent missing")?;
            if !parent_node.children.contains(&child) {
                parent_node.children.push(child);
            }
        }

        Ok(())
    }

    /// Change a component's parent.
    ///
    /// Equivalent to `detach_from_parent(child)` when `new_parent` is None.
    pub fn set_parent(&mut self, child: ComponentId, new_parent: Option<ComponentId>) -> Result<(), &'static str> {
        match new_parent {
            None => {
                self.detach_from_parent(child);
                Ok(())
            }
            Some(parent) => self.add_child(parent, child),
        }
    }

    /// Detach `child` from its current parent.
    ///
    /// This does *not* delete anything.
    pub fn detach_from_parent(&mut self, child: ComponentId) {
        let Some(old_parent) = self.parent_of(child) else {
            return;
        };

        // Clear child's parent.
        if let Some(node) = self.get_component_record_mut(child) {
            node.parent = None;
        }

        // Remove from old parent's children list.
        if let Some(parent_node) = self.get_component_record_mut(old_parent) {
            parent_node.children.retain(|&c| c != child);
        }
    }

    /// Remove a component from the world.
    ///
    /// This is a *leaf-only* removal: it fails if the component still has children.
    /// Use `remove_component_subtree` when you want to delete a whole branch.
    pub fn remove_component_leaf(&mut self, c: ComponentId) -> Result<(), &'static str> {
        let Some(node) = self.get_component_record(c) else {
            return Err("component does not exist");
        };
        if !node.children.is_empty() {
            return Err("component has children; use remove_component_subtree or detach children first");
        }

        self.detach_from_parent(c);
        self.components.remove(c);
        Ok(())
    }

    /// Remove a component and all its descendants.
    pub fn remove_component_subtree(&mut self, root: ComponentId) -> Result<(), &'static str> {
        if self.get_component_record(root).is_none() {
            return Err("component does not exist");
        }

        // Detach root first so parent doesn't retain dead child.
        self.detach_from_parent(root);

        // Post-order delete: collect subtree ids, then delete leaves upward.
        let mut stack = vec![root];
        let mut order: Vec<ComponentId> = Vec::new();
        while let Some(c) = stack.pop() {
            order.push(c);
            let children: Vec<ComponentId> = self.children_of(c).to_vec();
            for ch in children {
                stack.push(ch);
            }
        }

        // Delete in reverse (children first).
        for c in order.into_iter().rev() {
            // Clear parent/children links if node still exists.
            if let Some(node) = self.get_component_record_mut(c) {
                node.parent = None;
                node.children.clear();
            }
            self.components.remove(c);
        }

        Ok(())
    }
}
