pub mod camera_system;
pub mod renderable_system;
pub mod transform_system;
pub mod input_system;
pub mod system_world;
pub mod light_system;
pub mod lit_voxel_system;

pub use camera_system::{Camera3D, CameraHandle, CameraSystem};
pub use renderable_system::RenderableSystem;
pub use transform_system::TransformSystem;
pub use input_system::InputSystem;
pub use system_world::SystemWorld;
pub use light_system::LightSystem;
pub use lit_voxel_system::LitVoxelSystem;

use super::World;
use crate::engine::user_input::InputState;
use crate::engine::graphics::VisualWorld;

/// Individual system trait that processes specific component types.
///
/// This trait lives in `ecs/system/mod.rs` and is used by `SystemWorld` and all systems.
pub trait System: std::fmt::Debug {
    fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState, dt_sec: f32);
}
