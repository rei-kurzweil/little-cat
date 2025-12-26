use super::Component;
use crate::engine::ecs::entity::ComponentId;
use crate::engine::ecs::entity::EntityId;
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;
use crate::engine::graphics::primitives::Transform;

#[derive(Debug, Clone, Copy)]
pub struct TransformComponent {
    /// Engine-wide transform type (also used by renderer/VisualWorld).
    pub transform: Transform,
}

impl TransformComponent {
    pub fn new() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
    
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.transform.translation = [x, y, z];
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.transform.scale = [x, y, z];
        self
    }

    pub fn with_rotation_xyzw(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.transform.rotation = [x, y, z, w];
        self
    }
}

impl Component for TransformComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    // For now Transform doesn't need initialization.
    fn init(&mut self, _world: &World, _systems: &mut SystemWorld, _entity: EntityId, _component: ComponentId) {}
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self::new()
    }
}
