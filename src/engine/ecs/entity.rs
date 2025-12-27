use std::collections::HashMap;

pub type EntityId = u64;

/// Component identity scoped to a single entity.
pub type ComponentId = u32;

use crate::engine::ecs::component::Component;
use crate::engine::ecs::component::InstanceComponent;
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;

struct ComponentNode {
    component: Box<dyn Component>,
    parent: Option<ComponentId>,
    children: Vec<ComponentId>,
}

pub struct Entity {
    pub id: EntityId,

    next_component_id: ComponentId,

    /// Root component ids (a forest).
    roots: Vec<ComponentId>,

    /// Nodes stored by id (includes roots + all descendants).
    nodes: HashMap<ComponentId, ComponentNode>,

    /// Builder cursor: where `.with_component(...)` attaches.
    active_root: Option<ComponentId>,
}

impl core::fmt::Debug for Entity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Entity")
            .field("id", &self.id)
            .field("roots_len", &self.roots.len())
            .field("nodes_len", &self.nodes.len())
            .finish()
    }
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        let mut e = Self {
            id,
            next_component_id: 0,
            roots: Vec::new(),
            nodes: HashMap::new(),
            active_root: None,
        };

        // Default: every entity starts with one root InstanceComponent.
        let root = e.add_root(InstanceComponent::default());
        e.active_root = Some(root);

        e
    }

    pub fn alloc_component_id(&mut self) -> ComponentId {
        let id = self.next_component_id;
        self.next_component_id = self.next_component_id.wrapping_add(1);
        id
    }

    /// Add a new root component. Returns its ComponentId and makes it the active root.
    pub fn add_root(&mut self, c: impl Component + 'static) -> ComponentId {
        let cid = self.alloc_component_id();
        let mut boxed: Box<dyn Component> = Box::new(c);
        boxed.set_ids(self.id, cid);
        self.nodes.insert(
            cid,
            ComponentNode {
                component: boxed,
                parent: None,
                children: Vec::new(),
            },
        );
        self.roots.push(cid);
        self.active_root = Some(cid);
        cid
    }

    /// Builder-style: attach a component as a child of the active root (default root if none set).
    pub fn with_component(mut self, c: impl Component + 'static) -> Self {
        let parent = self
            .active_root
            .expect("Entity has no active root (should not happen; Entity::new adds one)");
        self.add_child(parent, c);
        self
    }

    /// Add a child component under a given parent id. Returns child ComponentId.
    pub fn add_child(&mut self, parent: ComponentId, c: impl Component + 'static) -> ComponentId {
        let cid = self.alloc_component_id();

        let mut boxed: Box<dyn Component> = Box::new(c);
        boxed.set_ids(self.id, cid);

        self.nodes.insert(
            cid,
            ComponentNode {
                component: boxed,
                parent: Some(parent),
                children: Vec::new(),
            },
        );

        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children.push(cid);
        } else {
            // If parent doesn't exist, we still inserted the node; that's a programmer error.
            // (Could also choose to remove it and return something.)
            panic!("add_child: parent ComponentId {parent} not found on entity {}", self.id);
        }

        cid
    }

    pub fn active_root(&self) -> Option<ComponentId> {
        self.active_root
    }

    pub fn set_active_root(&mut self, root: ComponentId) {
        // Optional: validate it's actually a root.
        if !self.roots.contains(&root) {
            panic!("set_active_root: ComponentId {root} is not a root on entity {}", self.id);
        }
        self.active_root = Some(root);
    }

    /// Insert a boxed component as a child under `parent`, returning the new id and a mutable ref to it.
    pub fn add_child_boxed_and_get_mut(
        &mut self,
        parent: ComponentId,
        c: Box<dyn Component>,
    ) -> (ComponentId, &mut Box<dyn Component>) {
        let cid = self.alloc_component_id();

        let mut c = c;
        c.set_ids(self.id, cid);

        self.nodes.insert(
            cid,
            ComponentNode {
                component: c,
                parent: Some(parent),
                children: Vec::new(),
            },
        );

        let Some(p) = self.nodes.get_mut(&parent) else {
            panic!("add_child_boxed_and_get_mut: parent ComponentId {parent} not found on entity {}", self.id);
        };
        p.children.push(cid);

        let node = self.nodes.get_mut(&cid).expect("just inserted node missing");
        (cid, &mut node.component)
    }

    /// Add a boxed component under the active root.
    pub fn add_under_active_root_boxed_and_get_mut(
        &mut self,
        c: Box<dyn Component>,
    ) -> (ComponentId, &mut Box<dyn Component>) {
        let parent = self
            .active_root
            .or_else(|| self.roots.first().copied())
            .expect("entity has no roots");
        self.add_child_boxed_and_get_mut(parent, c)
    }

    /// Roots (ids).
    pub fn roots(&self) -> &[ComponentId] {
        &self.roots
    }

    /// Iterate all nodes mutably (id + component).
    pub fn iter_components_mut(&mut self) -> impl Iterator<Item = (ComponentId, &mut Box<dyn Component>)> {
        self.nodes.iter_mut().map(|(id, node)| (*id, &mut node.component))
    }

    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        self.nodes.values().find_map(|node| {
            (node.component.as_ref() as &dyn std::any::Any).downcast_ref::<T>()
        })
    }

    pub fn get_component_with_id<T: 'static>(&self) -> Option<(ComponentId, &T)> {
        self.nodes.iter().find_map(|(id, node)| {
            (node.component.as_ref() as &dyn std::any::Any)
                .downcast_ref::<T>()
                .map(|t| (*id, t))
        })
    }

    pub fn has_component<T: 'static>(&self) -> bool {
        self.get_component::<T>().is_some()
    }

    /// Get a component by its ComponentId.
    pub fn get_component_by_id(&self, cid: ComponentId) -> Option<&Box<dyn Component>> {
        self.nodes.get(&cid).map(|node| &node.component)
    }

    /// Get a mutable component by its ComponentId.
    pub fn get_component_by_id_mut(&mut self, cid: ComponentId) -> Option<&mut Box<dyn Component>> {
        self.nodes.get_mut(&cid).map(|node| &mut node.component)
    }

    /// Get a component by its ComponentId and downcast to a specific type.
    pub fn get_component_by_id_as<T: 'static>(&self, cid: ComponentId) -> Option<&T> {
        self.nodes.get(&cid).and_then(|node| {
            (node.component.as_ref() as &dyn std::any::Any).downcast_ref::<T>()
        })
    }

    /// Get a mutable component by its ComponentId and downcast to a specific type.
    pub fn get_component_by_id_as_mut<T: 'static>(&mut self, cid: ComponentId) -> Option<&mut T> {
        self.nodes.get_mut(&cid).and_then(|node| {
            (node.component.as_mut() as &mut dyn std::any::Any).downcast_mut::<T>()
        })
    }

    /// Parent component id (None means it's a root).
    pub fn parent_of(&self, cid: ComponentId) -> Option<ComponentId> {
        self.nodes.get(&cid).and_then(|n| n.parent)
    }

    /// Get the parent component and downcast to a specific type.
    pub fn get_parent_as<T: 'static>(&self, cid: ComponentId) -> Option<(ComponentId, &T)> {
        let parent_id = self.parent_of(cid)?;
        self.get_component_by_id_as::<T>(parent_id).map(|c| (parent_id, c))
    }

    /// Get the parent component mutably and downcast to a specific type.
    pub fn get_parent_as_mut<T: 'static>(&mut self, cid: ComponentId) -> Option<(ComponentId, &mut T)> {
        let parent_id = self.parent_of(cid)?;
        self.get_component_by_id_as_mut::<T>(parent_id).map(|c| (parent_id, c))
    }

    /// Child component ids.
    pub fn children_of(&self, cid: ComponentId) -> &[ComponentId] {
        static EMPTY: [ComponentId; 0] = [];
        self.nodes.get(&cid).map(|n| n.children.as_slice()).unwrap_or(&EMPTY)
    }

    /// Initialize all components in root->child order.
    pub fn init_all(&mut self, world: &mut World, systems: &mut SystemWorld, visuals: &mut VisualWorld) {
        // Build a stable init order by walking from roots down.
        let mut order = Vec::<ComponentId>::new();
        let mut stack = Vec::<ComponentId>::new();

        for &r in self.roots.iter().rev() {
            stack.push(r);
        }

        while let Some(id) = stack.pop() {
            order.push(id);
            if let Some(node) = self.nodes.get(&id) {
                for &ch in node.children.iter().rev() {
                    stack.push(ch);
                }
            }
        }

        for cid in order {
            if let Some(node) = self.nodes.get_mut(&cid) {
                node.component.init(world, systems, visuals, self.id, cid);
            }
        }
    }

    /// Add a component as a child of the active root and initialize it immediately.
    pub fn add_component(
        &mut self,
        world: &mut World,
        systems: &mut SystemWorld,
        visuals: &mut VisualWorld,
        c: impl Component + 'static,
    ) -> ComponentId {
        let parent = self
            .active_root
            .or_else(|| self.roots.first().copied())
            .expect("entity has no roots");
        
        let cid = self.add_child(parent, c);
        
        // Initialize the component immediately
        if let Some(node) = self.nodes.get_mut(&cid) {
            node.component.set_ids(self.id, cid);
            node.component.init(world, systems, visuals, self.id, cid);
        }
        
        cid
    }

    /// Remove a component and call its cleanup.
    pub fn remove_component(
        &mut self,
        cid: ComponentId,
        world: &mut World,
        systems: &mut SystemWorld,
        visuals: &mut VisualWorld,
    ) -> bool {
        let Some(mut node) = self.nodes.remove(&cid) else {
            return false;
        };

        // Call cleanup on the component before removing
    node.component.cleanup(world, systems, visuals, self.id, cid);

        // Remove from parent's children list
        if let Some(parent_id) = node.parent {
            if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                parent_node.children.retain(|&c| c != cid);
            }
        } else {
            // It's a root, remove from roots
            self.roots.retain(|&r| r != cid);
        }

        // Recursively remove all children
        let children = node.children.clone();
        for child_id in children {
            self.remove_component(child_id, world, systems, visuals);
        }

        true
    }
}

