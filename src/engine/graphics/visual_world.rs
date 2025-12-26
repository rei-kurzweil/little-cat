use crate::engine::ecs::entity::{EntityId, ComponentId};
use crate::engine::ecs::{Renderable, Transform};
use crate::engine::graphics::primitives::InstanceHandle;

/// CPU-side per-entity instance payload (will become GPU instance-buffer data).
#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub transform: Transform,
}

impl From<Transform> for Instance {
    fn from(transform: Transform) -> Self {
        Self { transform }
    }
}

/// Renderer-friendly cache: flat list of (Renderable, Instance) ready for drawing.
/// Systems register/unregister instances here during component init/cleanup.
#[derive(Default)]
pub struct VisualWorld {
    /// Flat list of drawable instances, ready for rendering.
    instances: Vec<(Renderable, Instance)>,

    /// Next handle to allocate.
    next_handle: u32,

    /// Lookup map: InstanceHandle -> index in instances vec.
    /// This lets systems quickly find and update/remove specific instances.
    handle_to_index: std::collections::HashMap<InstanceHandle, usize>,

    /// Reverse lookup: (EntityId, ComponentId) -> InstanceHandle.
    /// Used for cleanup when entity/component is removed.
    component_to_handle: std::collections::HashMap<(EntityId, ComponentId), InstanceHandle>,
}

impl VisualWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.instances.clear();
        self.handle_to_index.clear();
        self.component_to_handle.clear();
        self.next_handle = 0;
    }

    /// Get all instances for rendering (just iterate the flat vec).
    pub fn instances(&self) -> &[(Renderable, Instance)] {
        &self.instances
    }

    /// Register a new instance. Returns the InstanceHandle that should be stored in InstanceComponent.
    /// Called by systems during component init.
    pub fn register(&mut self, id: EntityId, cid: ComponentId, renderable: Renderable, instance: Instance) -> InstanceHandle {
        let handle = InstanceHandle(self.next_handle);
        self.next_handle = self.next_handle.wrapping_add(1);

        let idx = self.instances.len();
        self.instances.push((renderable, instance));
        self.handle_to_index.insert(handle, idx);
        self.component_to_handle.insert((id, cid), handle);

        handle
    }

    /// Remove an instance by handle. Called by systems during component cleanup.
    /// Returns true if the instance was found and removed.
    pub fn remove(&mut self, handle: InstanceHandle) -> bool {
        if let Some(idx) = self.handle_to_index.remove(&handle) {
            // Swap-remove from instances vec
            self.instances.swap_remove(idx);

            // Fix up the lookup for the swapped element (if any)
            if idx < self.instances.len() {
                // Find which handle was at the end and update its index
                if let Some((moved_handle, _)) = self.handle_to_index.iter()
                    .find(|(_, i)| **i == self.instances.len())
                {
                    self.handle_to_index.insert(*moved_handle, idx);
                }
            }

            // Remove from component_to_handle reverse lookup
            self.component_to_handle.retain(|_, &mut h| h != handle);

            true
        } else {
            false
        }
    }

    /// Remove an instance by (EntityId, ComponentId). Useful for cleanup.
    pub fn remove_by_component(&mut self, id: EntityId, cid: ComponentId) -> bool {
        if let Some(&handle) = self.component_to_handle.get(&(id, cid)) {
            self.remove(handle)
        } else {
            false
        }
    }

    /// Remove all instances for an entity. Called when entity is removed from world.
    pub fn remove_entity(&mut self, id: EntityId) {
        // Collect all component ids for this entity
        let handles: Vec<InstanceHandle> = self.component_to_handle.iter()
            .filter(|((eid, _), _)| *eid == id)
            .map(|(_, &handle)| handle)
            .collect();

        for handle in handles {
            self.remove(handle);
        }
    }

    /// Update just the transform of an existing instance by handle.
    pub fn update_transform(&mut self, handle: InstanceHandle, transform: Transform) -> bool {
        if let Some(&idx) = self.handle_to_index.get(&handle) {
            self.instances[idx].1.transform = transform;
            true
        } else {
            false
        }
    }

    /// Update the renderable and instance data by handle.
    pub fn update(&mut self, handle: InstanceHandle, renderable: Renderable, instance: Instance) -> bool {
        if let Some(&idx) = self.handle_to_index.get(&handle) {
            self.instances[idx] = (renderable, instance);
            true
        } else {
            false
        }
    }
}