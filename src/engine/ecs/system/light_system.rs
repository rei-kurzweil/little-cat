use crate::engine::ecs::component::PointLightComponent;
use crate::engine::ecs::system::System;
use crate::engine::ecs::system::TransformSystem;
use crate::engine::ecs::{ComponentId, World};
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;

/// ECS lighting system.
///
/// Keeps `VisualWorld`'s point-light list in sync with ECS.
#[derive(Debug, Default)]
pub struct LightSystem;

impl LightSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn register_light(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        let Some(light) = world.get_component_by_id_as::<PointLightComponent>(component) else {
            return;
        };

        let position_ws = TransformSystem::world_position(world, component).unwrap_or([0.0, 0.0, 0.0]);

        visuals.upsert_point_light(
            component,
            crate::engine::graphics::visual_world::VisualPointLight {
                position_ws,
                intensity: light.intensity,
                distance: light.distance,
                color: light.color,
            },
        );
    }

    /// Called when a TransformComponent changes.
    ///
    /// Updates all descendant point lights' positions in `VisualWorld`.
    pub fn transform_changed(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        let debug = std::env::var_os("LC_DEBUG_LIGHTS").is_some();
        if debug {
            println!("[LightSystem] transform_changed root={:?}", component);
        }

        let mut visited_nodes = 0usize;
        let mut updated_lights = 0usize;

        let mut stack = vec![component];
        while let Some(node) = stack.pop() {
            visited_nodes += 1;
            for &child in world.children_of(node) {
                stack.push(child);
                if let Some(light) = world.get_component_by_id_as::<PointLightComponent>(child) {
                    let position_ws =
                        TransformSystem::world_position(world, child).unwrap_or([0.0, 0.0, 0.0]);
                    updated_lights += 1;

                    if debug {
                        println!(
                            "[LightSystem]  -> update light={:?} pos={:?} intensity={} distance={} color={:?}",
                            child, position_ws, light.intensity, light.distance, light.color
                        );
                    }
                    
                    visuals.upsert_point_light(
                        child,
                        crate::engine::graphics::visual_world::VisualPointLight {
                            position_ws,
                            intensity: light.intensity,
                            distance: light.distance,
                            color: light.color,
                        },
                    );
                }
            }
        }

        if debug {
            println!(
                "[LightSystem] transform_changed done visited_nodes={} updated_lights={}",
                visited_nodes, updated_lights
            );
        }
    }
}

impl System for LightSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState, _dt_sec: f32) {
        // No-op for now.
    }
}
