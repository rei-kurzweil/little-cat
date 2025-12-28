use crate::engine::ecs::entity::{EntityId, ComponentId};
use crate::engine::ecs::Transform;
use crate::engine::graphics::GpuRenderable;
use crate::engine::graphics::primitives::InstanceHandle;

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub transform: Transform,
}

impl From<Transform> for Instance {
    fn from(transform: Transform) -> Self {
        Self { transform }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DrawBatch {
    pub material: crate::engine::graphics::MaterialHandle,
    pub mesh: crate::engine::graphics::primitives::MeshHandle,
    /// Range into `draw_order`
    pub start: usize,
    pub count: usize,
}

#[derive(Default)]
pub struct VisualWorld {
    instances: Vec<(GpuRenderable, Instance)>,

    next_handle: u32,
    handle_to_index: std::collections::HashMap<InstanceHandle, usize>,
    component_to_handle: std::collections::HashMap<(EntityId, ComponentId), InstanceHandle>,

    // Cached draw data (rebuilt when dirty)
    dirty_draw_cache: bool,
    /// True when per-instance data (e.g. model matrices) changed and any cached GPU instance
    /// buffer should be rebuilt/uploaded.
    dirty_instance_data: bool,
    draw_order: Vec<u32>,     // indices into `instances`
    draw_batches: Vec<DrawBatch>,
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

        self.dirty_draw_cache = true;
        self.dirty_instance_data = true;
        self.draw_order.clear();
        self.draw_batches.clear();
    }

    /// Returns whether any per-instance data has changed since the last time it was consumed.
    pub fn instance_data_dirty(&self) -> bool {
        self.dirty_instance_data
    }

    /// Consume the instance-data dirty flag.
    pub fn take_instance_data_dirty(&mut self) -> bool {
        let v = self.dirty_instance_data;
        self.dirty_instance_data = false;
        v
    }

    pub fn instances(&self) -> &[(GpuRenderable, Instance)] {
        &self.instances
    }

    /// Indices into `instances()` in the order they should be drawn (opaque batching).
    pub fn draw_order(&self) -> &[u32] {
        &self.draw_order
    }

    pub fn draw_batches(&self) -> &[DrawBatch] {
        &self.draw_batches
    }

    /// Call once per frame before rendering. Cheap if nothing changed.
    ///
    /// Returns `true` if the cached draw order/batches were rebuilt this call.
    pub fn prepare_draw_cache(&mut self) -> bool {
        if !self.dirty_draw_cache {
            return false;
        }

        self.draw_order.clear();
    self.draw_order.extend(0..self.instances.len() as u32);

        // Sort by (material, mesh). Stable sort keeps relative order for identical keys.
        self.draw_order.sort_by_key(|&i| {
            let (r, _inst) = self.instances[i as usize];
            // pack into u64: material in high bits, mesh in low bits
            ((r.material.0 as u64) << 32) | (r.mesh.0 as u64)
        });

        self.draw_batches.clear();
        let mut cursor = 0usize;
        while cursor < self.draw_order.len() {
            let idx0 = self.draw_order[cursor] as usize;
            let (r0, _) = self.instances[idx0];
            let material = r0.material;
            let mesh = r0.mesh;

            let start = cursor;
            cursor += 1;

            while cursor < self.draw_order.len() {
                let idx = self.draw_order[cursor] as usize;
                let (r, _) = self.instances[idx];
                if r.material == material && r.mesh == mesh {
                    cursor += 1;
                } else {
                    break;
                }
            }

            self.draw_batches.push(DrawBatch {
                material,
                mesh,
                start,
                count: cursor - start,
            });
        }

        self.dirty_draw_cache = false;
        true
    }

    pub fn register(
        &mut self,
        id: EntityId,
        cid: ComponentId,
        renderable: GpuRenderable,
        instance: Instance,
    ) -> InstanceHandle {
        let handle = InstanceHandle(self.next_handle);
        self.next_handle = self.next_handle.wrapping_add(1);

        let idx = self.instances.len();
        self.instances.push((renderable, instance));
        self.handle_to_index.insert(handle, idx);
        self.component_to_handle.insert((id, cid), handle);

        self.dirty_draw_cache = true;
        self.dirty_instance_data = true;
        handle
    }

    pub fn remove(&mut self, handle: InstanceHandle) -> bool {
        if let Some(idx) = self.handle_to_index.remove(&handle) {
            self.instances.swap_remove(idx);

            if idx < self.instances.len() {
                // NOTE: This is O(n). Consider storing index->handle too if it becomes hot.
                if let Some((moved_handle, _)) = self
                    .handle_to_index
                    .iter()
                    .find(|(_, i)| **i == self.instances.len())
                {
                    self.handle_to_index.insert(*moved_handle, idx);
                }
            }

            self.component_to_handle.retain(|_, &mut h| h != handle);

            self.dirty_draw_cache = true;
            self.dirty_instance_data = true;
            true
        } else {
            false
        }
    }

    pub fn update_transform(&mut self, handle: InstanceHandle, transform: Transform) -> bool {
        if let Some(&idx) = self.handle_to_index.get(&handle) {
            self.instances[idx].1.transform = transform;
            self.dirty_instance_data = true;
            // transform-only doesn’t affect batching by (material, mesh)
            true
        } else {
            false
        }
    }

    pub fn update_model(&mut self, handle: InstanceHandle, model: [[f32; 4]; 4]) -> bool {
        if let Some(&idx) = self.handle_to_index.get(&handle) {
            self.instances[idx].1.transform.model = model;
            self.dirty_instance_data = true;
            // model-only doesn’t affect batching by (material, mesh)
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, handle: InstanceHandle, renderable: GpuRenderable, instance: Instance) -> bool {
        if let Some(&idx) = self.handle_to_index.get(&handle) {
            self.instances[idx] = (renderable, instance);
            self.dirty_draw_cache = true; // renderable changes likely affect sort/batch
            self.dirty_instance_data = true;
            true
        } else {
            false
        }
    }
}