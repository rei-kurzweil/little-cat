use crate::engine::ecs::component::{InstanceComponent, RenderableComponent, TransformComponent};
use crate::engine::ecs::entity::ComponentId;
use crate::engine::ecs::entity::EntityId;
use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::{GpuRenderable, Instance, VisualWorld};
use crate::engine::user_input::InputState;

/// System that registers/updates renderables in the `VisualWorld`.
///
/// Contract / intent:
/// - A `RenderableComponent` must be a *child* of an `InstanceComponent`.
/// - **Each `InstanceComponent` corresponds to exactly one `VisualWorld` `Instance`.**
///   Multiple renderable children under the same `InstanceComponent` share that one instance.
/// - The `InstanceComponent` stores the `InstanceHandle` which is assigned when the first
///   renderable child registers.
/// - Optional: if there is a sibling `TransformComponent` under the same `InstanceComponent`,
///   we use it as the instance transform. Otherwise we fall back to `Transform::default()`.
#[derive(Debug, Default)]
pub struct RenderableSystem {
    renderables: Vec<(EntityId, ComponentId)>,
}

impl RenderableSystem {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
        }
    }

    /// Register a renderable component with this system.
    ///
    /// This is also where we ensure a `VisualWorld` instance exists for it.
    pub fn register_renderable(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        entity: EntityId,
        component: ComponentId,
    ) {
        if !self
            .renderables
            .iter()
            .any(|(e, c)| *e == entity && *c == component)
        {
            self.renderables.push((entity, component));
        }

        let Some(ent) = world.get_entity_mut(entity) else {
            return;
        };

        // Pull immutable data first.
        let gpu_r = {
            let Some(renderable_comp) = ent.get_component_by_id_as::<RenderableComponent>(component)
            else {
                return;
            };
            Self::to_gpu_renderable(*renderable_comp)
        };
        // Each InstanceComponent (and its immediate children) defines a VisualWorld Instance.
        // Renderables may be nested under other components; we walk up to find the nearest
        // ancestor InstanceComponent.
        let instance_cid = {
            let mut cur = component;
            loop {
                let Some(parent) = ent.parent_of(cur) else {
                    return;
                };
                if ent.get_component_by_id_as::<InstanceComponent>(parent).is_some() {
                    break parent;
                }
                cur = parent;
            }
        };

        // First TransformComponent directly under the InstanceComponent (if present).
        let transform_comp = ent
            .children_of(instance_cid)
            .iter()
            .find_map(|&cid| ent.get_component_by_id_as::<TransformComponent>(cid));
        let transform = if let Some(t) = transform_comp {
            t.transform
        } else {
            crate::engine::graphics::primitives::Transform::default()
        };
        let inst = Instance { transform };

        // Now mutably borrow the InstanceComponent to store the handle.
        let Some(instance_comp) = ent.get_component_by_id_as_mut::<InstanceComponent>(instance_cid) else {
            return;
        };

        // If it's already registered, nothing else to do.
        if instance_comp.get_handle().is_some() {
            return;
        }

        let handle = visuals.register(entity, instance_cid, gpu_r, inst);
        instance_comp.handle = Some(handle);
    }

    fn to_gpu_renderable(rc: RenderableComponent) -> GpuRenderable {
        GpuRenderable {
            mesh: rc.renderable.mesh,
            material: rc.renderable.material,
        }
    }
}

impl System for RenderableSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState) {
        // Intentionally a no-op for now.
        //
        // Per your architecture: VisualWorld registration happens at component registration time
        // (RenderableComponent::init -> SystemWorld::register_renderable -> RenderableSystem::register_renderable).
        //
        // Later, tick() can be used for per-frame sync (transform updates, material changes, etc.)
        // once we decide how to represent those components and what events/dirty flags we have.
    }
}
