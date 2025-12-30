use crate::engine::ecs::component::{InputComponent, TransformComponent, Camera2DComponent, InstanceComponent};
use crate::engine::ecs::ComponentId;
use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;
use winit::keyboard::Key;

/// System that processes input components and updates transforms or cameras based on WASD input.
#[derive(Debug, Default)]
pub struct InputSystem {
    inputs: Vec<ComponentId>,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
        }
    }

    /// Register an InputComponent.
    pub fn register_input(&mut self, component: ComponentId) {
        if !self.inputs.iter().any(|c| *c == component) {
            self.inputs.push(component);
        }
    }

    /// Process input and update transforms/cameras. Takes command queue to queue updates.
    pub fn process_input(
        &mut self,
        world: &mut World,
        input: &InputState,
        queue: &mut crate::engine::ecs::CommandQueue,
        dt_sec: f32,
    ) {
        
        // Check for WASD keys using Character variant
        let w = input.key_down(&Key::Character("w".into())) || input.key_down(&Key::Character("W".into()));
        let a = input.key_down(&Key::Character("a".into())) || input.key_down(&Key::Character("A".into()));
        let s = input.key_down(&Key::Character("s".into())) || input.key_down(&Key::Character("S".into()));
        let d = input.key_down(&Key::Character("d".into())) || input.key_down(&Key::Character("D".into()));

        // Debug: print key states
        if w || a || s || d {
            let mut keys = Vec::new();
            if w { keys.push("W"); }
            if a { keys.push("A"); }
            if s { keys.push("S"); }
            if d { keys.push("D"); }
            //println!("[InputSystem] Keys pressed: {}", keys.join(", "));
        }

        if !w && !a && !s && !d {
            return; // No movement keys pressed
        }

        // Calculate movement delta
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        if w { dy -= 1.0; }
        if s { dy += 1.0; }
        if a { dx -= 1.0; }
        if d { dx += 1.0; }

        // Normalize diagonal movement
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            dx /= len;
            dy /= len;
        }

        
        for &input_cid in &self.inputs {
            let Some(input_comp) = world.get_component_by_id_as::<InputComponent>(input_cid) else {
                println!("[InputSystem] Input component {:?} not found", input_cid);
                continue;
            };
            let speed = input_comp.speed * dt_sec; // Scale by delta time

            // Check parent hierarchy
            let Some(parent) = world.parent_of(input_cid) else {
                println!("[InputSystem] Input component {:?} has no parent", input_cid);
                continue;
            };

            // Check if parent is TransformComponent
            if let Some(transform_comp) = world.get_component_by_id_as::<TransformComponent>(parent) {
                let transform_parent = world.parent_of(parent);
                
                // Case 1: TransformComponent -> InstanceComponent (normal case)
                if let Some(grandparent) = transform_parent {
                    if world.get_component_by_id_as::<InstanceComponent>(grandparent).is_some() {
                        println!("[InputSystem] Updating InstanceComponent via TransformComponent (dx={:.3}, dy={:.3}, speed={:.3})", dx * speed, dy * speed, speed);
                        // Update TransformComponent and queue update command
                        if let Some(transform_comp_mut) = world.get_component_by_id_as_mut::<TransformComponent>(parent) {
                            transform_comp_mut.transform.translation[0] += dx * speed;
                            transform_comp_mut.transform.translation[1] += dy * speed;
                            transform_comp_mut.transform.recompute_model();
                            // Queue update command - will be processed after tick
                            queue.queue_update_transform(parent, transform_comp_mut.transform);
                        }
                    }
                    // Case 2: TransformComponent -> Camera2DComponent (camera case)
                    else if world.get_component_by_id_as::<Camera2DComponent>(grandparent).is_some() {
                        // Update Camera2DComponent's TransformComponent directly
                        // CameraSystem will pick this up in the same tick
                        if let Some(transform_comp_mut) = world.get_component_by_id_as_mut::<TransformComponent>(parent) {
                            transform_comp_mut.transform.translation[0] += dx * speed;
                            transform_comp_mut.transform.translation[1] += dy * speed;
                            transform_comp_mut.transform.recompute_model();
                            // No need to queue - CameraSystem reads it directly in tick
                        }
                    } else {
                        println!("[InputSystem] TransformComponent parent {:?} is neither InstanceComponent nor Camera2DComponent", grandparent);
                    }
                } else {
                    println!("[InputSystem] TransformComponent {:?} has no parent", parent);
                }
            } else {
                println!("[InputSystem] Input component {:?} parent {:?} is not a TransformComponent", input_cid, parent);
            }
        }
    }
}

impl System for InputSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState, _dt_sec: f32) {
        // InputSystem processes input via process_input which takes command queue
        // This tick is a no-op since we need command queue access
    }
}
