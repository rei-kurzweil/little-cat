use crate::engine::ecs::component::{InputComponent, TransformComponent};
use crate::engine::ecs::ComponentId;
use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;
use winit::keyboard::Key;

/// System that processes input components and updates transforms based on WASD input.
///
/// Intended topology (simple one-way data flow):
/// InputComponent -> TransformComponent -> (Camera2DComponent, RenderableComponent, ...)
#[derive(Debug, Default)]
pub struct InputSystem {
    inputs: Vec<ComponentId>,
}

impl InputSystem {
    pub fn new() -> Self {
        Self { inputs: Vec::new() }
    }

    /// Register an InputComponent.
    pub fn register_input(&mut self, component: ComponentId) {
        if !self.inputs.iter().any(|c| *c == component) {
            self.inputs.push(component);
        }
    }

    /// Process input and queue at most one transform update per InputComponent.
    ///
    /// This only supports the intended topology:
    /// InputComponent -> TransformComponent (child)
    pub fn process_input(
        &mut self,
        world: &mut World,
        input: &InputState,
        queue: &mut crate::engine::ecs::CommandQueue,
        dt_sec: f32,
    ) {
        // Read WASD.
        let w = input.key_down(&Key::Character("w".into()))
            || input.key_down(&Key::Character("W".into()));
        let a = input.key_down(&Key::Character("a".into()))
            || input.key_down(&Key::Character("A".into()));
        let s = input.key_down(&Key::Character("s".into()))
            || input.key_down(&Key::Character("S".into()));
        let d = input.key_down(&Key::Character("d".into()))
            || input.key_down(&Key::Character("D".into()));

        if !w && !a && !s && !d {
            return;
        }

        // Movement delta.
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        if w {
            dy -= 1.0;
        }
        if s {
            dy += 1.0;
        }
        if a {
            dx -= 1.0;
        }
        if d {
            dx += 1.0;
        }

        // Normalize diagonal movement.
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            dx /= len;
            dy /= len;
        }

        for &input_cid in &self.inputs {
            let Some(input_comp) = world.get_component_by_id_as::<InputComponent>(input_cid) else {
                continue;
            };
            let speed = input_comp.speed * dt_sec;

            // Find TransformComponent child.
            let transform_child = world
                .children_of(input_cid)
                .iter()
                .copied()
                .find(|&cid| world.get_component_by_id_as::<TransformComponent>(cid).is_some());

            let Some(transform_cid) = transform_child else {
                continue;
            };

            if let Some(transform_comp_mut) =
                world.get_component_by_id_as_mut::<TransformComponent>(transform_cid)
            {
                transform_comp_mut.transform.translation[0] += dx * speed;
                transform_comp_mut.transform.translation[1] += dy * speed;
                transform_comp_mut.transform.recompute_model();
                queue.queue_update_transform(transform_cid, transform_comp_mut.transform);
            }
        }
    }
}

impl System for InputSystem {
    fn tick(
        &mut self,
        _world: &mut World,
        _visuals: &mut VisualWorld,
        _input: &InputState,
        _dt_sec: f32,
    ) {
        // InputSystem is driven by SystemWorld::tick calling process_input with a CommandQueue.
    }
}
