use crate::engine::ecs::component::{InstanceComponent, TransformComponent};
use crate::engine::ecs::entity::ComponentId;
use crate::engine::ecs::entity::EntityId;
use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;

/// System responsible for syncing `TransformComponent` changes into `VisualWorld`.
///
/// Key points:
/// - An entity can have multiple TransformComponents.
/// - A TransformComponent should be a child of an InstanceComponent.
/// - InstanceComponent owns the `InstanceHandle` pointing into VisualWorld.
#[derive(Debug, Default)]
pub struct TransformSystem;

impl TransformSystem {
    pub fn new() -> Self {
        Self
    }

    /// Called by TransformComponent when its values change.
    ///
    /// This updates the transform of the VisualWorld instance corresponding to the *parent*
    /// InstanceComponent of this TransformComponent.
    pub fn transform_changed(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        entity: EntityId,
        component: ComponentId,
    ) {
        let Some(ent) = world.get_entity_mut(entity) else {
            return;
        };

        // Each InstanceComponent (and its immediate children) defines a VisualWorld Instance.
        // TransformComponents may be nested under other components; we walk up to find the nearest
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

        let Some(instance_comp) = ent.get_component_by_id_as::<InstanceComponent>(instance_cid) else {
            return;
        };

        let Some(handle) = instance_comp.get_handle() else {
            // Visual instance not registered yet (RenderableSystem likely hasn't run).
            return;
        };

        let Some(transform_comp) = ent.get_component_by_id_as::<TransformComponent>(component) else {
            return;
        };

        visuals.update_model(handle, transform_comp.transform.model);
    }
}

impl System for TransformSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState) {
        // No-op. Transform updates are event-driven via `transform_changed`.
    }
}
