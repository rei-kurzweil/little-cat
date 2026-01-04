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

    fn compute_transform(
        &self,
        speed_units_per_sec: f32,
        input: &InputState,
        dt_sec: f32,
        transform: &mut crate::engine::graphics::primitives::Transform,
    ) {
        // Read movement keys.
        let w = input.key_down(&Key::Character("w".into()))
            || input.key_down(&Key::Character("W".into()));
        let a = input.key_down(&Key::Character("a".into()))
            || input.key_down(&Key::Character("A".into()));
        let s = input.key_down(&Key::Character("s".into()))
            || input.key_down(&Key::Character("S".into()));
        let d = input.key_down(&Key::Character("d".into()))
            || input.key_down(&Key::Character("D".into()));

        // Roll keys.
        let q = input.key_down(&Key::Character("q".into()))
            || input.key_down(&Key::Character("Q".into()));
        let e = input.key_down(&Key::Character("e".into()))
            || input.key_down(&Key::Character("E".into()));

        // Translation delta.
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

        let speed = speed_units_per_sec * dt_sec;
        transform.translation[0] += dx * speed;
        transform.translation[1] += dy * speed;

        // Roll around Z.
        if q || e {
            const ROT_SPEED_RAD_PER_SEC: f32 = 1.5;
            let dir = (q as i32) as f32 - (e as i32) as f32;
            let dtheta = dir * ROT_SPEED_RAD_PER_SEC * dt_sec;
            let (sz, cz) = (0.5 * dtheta).sin_cos();
            let qz = [0.0f32, 0.0f32, sz, cz];

            fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
                let (ax, ay, az, aw) = (a[0], a[1], a[2], a[3]);
                let (bx, by, bz, bw) = (b[0], b[1], b[2], b[3]);
                [
                    aw * bx + ax * bw + ay * bz - az * by,
                    aw * by - ax * bz + ay * bw + az * bx,
                    aw * bz + ax * by - ay * bx + az * bw,
                    aw * bw - ax * bx - ay * by - az * bz,
                ]
            }

            // Apply local Z-roll increment.
            transform.rotation = quat_mul(transform.rotation, qz);
        }

        transform.recompute_model();
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
        // We gate early to avoid scanning inputs if nothing relevant is pressed.
        let any_move = input.key_down(&Key::Character("w".into()))
            || input.key_down(&Key::Character("W".into()))
            || input.key_down(&Key::Character("a".into()))
            || input.key_down(&Key::Character("A".into()))
            || input.key_down(&Key::Character("s".into()))
            || input.key_down(&Key::Character("S".into()))
            || input.key_down(&Key::Character("d".into()))
            || input.key_down(&Key::Character("D".into()))
            || input.key_down(&Key::Character("q".into()))
            || input.key_down(&Key::Character("Q".into()))
            || input.key_down(&Key::Character("e".into()))
            || input.key_down(&Key::Character("E".into()));

        if !any_move {
            return;
        }

        for &input_cid in &self.inputs {
            let speed_units_per_sec = match world.get_component_by_id_as::<InputComponent>(input_cid) {
                Some(input_comp) => input_comp.speed,
                None => continue,
            };

            // Find TransformComponent child. If absent, we don't compute.
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
                self.compute_transform(speed_units_per_sec, input, dt_sec, &mut transform_comp_mut.transform);
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
