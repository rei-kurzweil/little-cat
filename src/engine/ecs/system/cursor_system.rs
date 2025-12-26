use crate::engine::ecs::component::InstanceComponent;
use crate::engine::ecs::entity::{ComponentId, EntityId};
use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;
use crate::engine::graphics::primitives::Transform;
use crate::engine::user_input::InputState;

/// System that updates cursor-following entities.
#[derive(Debug, Default)]
pub struct CursorSystem {
    cursors: Vec<(EntityId, ComponentId)>,
}

impl CursorSystem {
    pub fn new() -> Self {
        Self {
            cursors: Vec::new(),
        }
    }

    /// Register an entity as being cursor-driven.
    ///
    /// For now this stores just the entity id. Later this can store component pointers/handles
    /// once ECS storage is component-centric.
    pub fn register_cursor(&mut self, entity: EntityId, component: ComponentId) {
        if !self.cursors.iter().any(|(e, c)| *e == entity && *c == component) {
            self.cursors.push((entity, component));
        }
    }

    pub fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState) {
        <Self as System>::tick(self, world, visuals, input)
    }
}

impl System for CursorSystem {
    fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState) {
        // Source of truth is `CursorComponent::init` registration.
        // If nothing registered yet, do nothing.
        if self.cursors.is_empty() {
            return;
        }

        let Some(cursor_pos) = input.cursor_pos else {
            return;
        };

        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        // TODO: use actual window size / camera projection.
        let ndc_x = (cursor_pos.0 / 800.0) * 2.0 - 1.0;
        let ndc_y = 1.0 - (cursor_pos.1 / 600.0) * 2.0;

        // For each registered cursor component, find its parent InstanceComponent
        // and update the transform in the visual world.
        for (entity_id, cursor_cid) in self.cursors.iter().copied() {
            let Some(entity) = world.get_entity_mut(entity_id) else {
                continue;
            };

            // Get the parent component and verify it's an InstanceComponent
            if let Some((_parent_cid, instance_comp)) = entity.get_parent_as::<InstanceComponent>(cursor_cid) {
                // Get the instance handle from the parent InstanceComponent
                if let Some(handle) = instance_comp.get_handle() {
                    // Update the transform in the visual world using the handle
                    let mut transform = Transform::default();
                    transform.translation = [ndc_x, ndc_y, 0.0];
                    transform.scale = [0.1, 0.1, 1.0];

                    visuals.update_transform(handle, transform);
                }
            }
        }
    }
}