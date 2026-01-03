use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;

/// ECS lighting system.
///
/// Placeholder: currently does nothing. Later it can gather light components and
/// upload a GPU light buffer for the renderer.
#[derive(Debug, Default)]
pub struct LightSystem;

impl LightSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for LightSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState, _dt_sec: f32) {
        // No-op for now.
    }
}
