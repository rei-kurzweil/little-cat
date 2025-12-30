pub mod cursor_system;
pub mod camera_system;
pub mod renderable_system;
pub mod transform_system;
pub mod input_system;
pub mod system_world;

pub use cursor_system::CursorSystem;
pub use camera_system::{Camera, CameraHandle, CameraSystem};
pub use renderable_system::RenderableSystem;
pub use transform_system::TransformSystem;
pub use input_system::InputSystem;
pub use system_world::SystemWorld;

use super::World;
use crate::engine::user_input::InputState;
use crate::engine::graphics::VisualWorld;

/// Individual system trait that processes specific component types.
///
/// This trait lives in `ecs/system/mod.rs` and is used by `SystemWorld` and all systems.
pub trait System: std::fmt::Debug {
    fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState, dt_sec: f32);
}
