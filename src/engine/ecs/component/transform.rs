use crate::engine::ecs::component::Component;
use crate::engine::graphics::primitives::Transform;

#[derive(Debug, Clone, Copy)]
pub struct TransformComponent {
    pub transform: Transform,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self { transform: Transform::default() }
    }
}

impl Component for TransformComponent {}
