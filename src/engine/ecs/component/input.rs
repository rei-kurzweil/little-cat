use crate::engine::ecs::ComponentId;
use crate::engine::ecs::component::Component;

/// Input component that responds to keyboard input (WASD).
#[derive(Debug, Clone, Default)]
pub struct InputComponent {
    pub speed: f32,
}

impl InputComponent {
    pub fn new() -> Self {
        Self { speed: 0.01 }
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

impl Component for InputComponent {
    fn name(&self) -> &'static str {
        "input"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self, queue: &mut crate::engine::ecs::CommandQueue, component: ComponentId) {
        queue.queue_register_input(component);
    }
}
