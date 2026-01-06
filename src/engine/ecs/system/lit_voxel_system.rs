use crate::engine::ecs::World;
use crate::engine::ecs::system::System;
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;

/// CPU-side voxel lighting/shading system.
///
/// Placeholder: eventually this will compute per-instance shade/emissive data
/// (e.g. skylight occlusion) and upload it to a GPU storage buffer.
#[derive(Debug, Default)]
pub struct LitVoxelSystem;

impl LitVoxelSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for LitVoxelSystem {
    fn tick(
        &mut self,
        _world: &mut World,
        _visuals: &mut VisualWorld,
        _input: &InputState,
        _dt_sec: f32,
    ) {
        // No-op for now.
    }
}
