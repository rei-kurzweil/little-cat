use std::collections::HashMap;

use crate::engine::ecs::entity::EntityId;
use crate::engine::ecs::{Renderable, Transform};

/// CPU-side per-entity instance payload (will become GPU instance-buffer data).
/// Intentionally excludes uniforms (those belong to Renderable/material in your model).
#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub transform: Transform,
    // Future:
    // pub vertex_attribs: VertexAttributesInstance,
    // pub flags: InstanceFlags,
}

impl From<Transform> for Instance {
    fn from(transform: Transform) -> Self {
        Self { transform }
    }
}

/// Renderer-friendly cache, organized for instanced draws.
/// Groups instances by Renderable (mesh+material/pipeline).
#[derive(Default)]
pub struct VisualWorld {
    groups: HashMap<Renderable, Vec<(EntityId, Instance)>>,
}

impl VisualWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.groups.clear();
    }

    pub fn groups(&self) -> &HashMap<Renderable, Vec<(EntityId, Instance)>> {
        &self.groups
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        for (_renderable, list) in self.groups.iter_mut() {
            if let Some(pos) = list.iter().position(|(eid, _)| *eid == id) {
                list.swap_remove(pos);
            }
        }
        self.groups.retain(|_, v| !v.is_empty());
    }

    pub fn upsert(&mut self, id: EntityId, renderable: Renderable, instance: Instance) {
        let list = self.groups.entry(renderable).or_default();

        // Simple O(n) update for now; later add an EntityId->(Renderable, index) map for O(1).
        if let Some((_eid, slot)) = list.iter_mut().find(|(eid, _)| *eid == id) {
            *slot = instance;
        } else {
            list.push((id, instance));
        }
    }

    pub fn extend_from_iter(
        &mut self,
        iter: impl IntoIterator<Item = (EntityId, Transform, Renderable)>,
    ) {
        for (id, transform, renderable) in iter {
            self.upsert(id, renderable, Instance::from(transform));
        }
    }
}