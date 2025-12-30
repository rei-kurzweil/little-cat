use crate::engine::ecs::component::{InstanceComponent, TransformComponent, Camera2DComponent};
use crate::engine::ecs::ComponentId;
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
    /// InstanceComponent of this TransformComponent, or updates camera translation if the
    /// transform is a child of a Camera2DComponent.
    pub fn transform_changed(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
        camera_system: &mut crate::engine::ecs::system::CameraSystem,
    ) {
        // Check if this transform is a child of a Camera2DComponent
        let parent = world.parent_of(component);
        if let Some(parent_id) = parent {
            if world.get_component_by_id_as::<Camera2DComponent>(parent_id).is_some() {
                // This transform is part of a Camera2D - update camera translation
                camera_system.update_camera_2d_from_transform(world, visuals, component);
                return; // Don't update VisualWorld instance for camera transforms
            }
        }

        let Some(transform_comp) = world.get_component_by_id_as::<TransformComponent>(component) else {
            return;
        };

        // Normal case: Each InstanceComponent (and its immediate children) defines a VisualWorld Instance.
        // TransformComponents may be nested under other components; we walk up to find the nearest
        // ancestor InstanceComponent.
        let instance_cid = {
            let mut cur = component;
            loop {
                let Some(parent) = world.parent_of(cur) else {
                    return;
                };
                if world.get_component_by_id_as::<InstanceComponent>(parent).is_some() {
                    break parent;
                }
                cur = parent;
            }
        };

        let Some(instance_comp) = world.get_component_by_id_as::<InstanceComponent>(instance_cid) else {
            return;
        };

        let Some(handle) = instance_comp.get_handle() else {
            // Visual instance not registered yet (RenderableSystem likely hasn't run).
            return;
        };

        visuals.update_model(handle, transform_comp.transform.model);
    }
}

impl System for TransformSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState, _dt_sec: f32) {
        // No-op. Transform updates are event-driven via `transform_changed`.
    }
}
