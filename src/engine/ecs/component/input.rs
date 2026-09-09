use crate::engine::ecs::ComponentId;
use crate::engine::ecs::component::Component;

/// Input component that responds to keyboard input (WASD).
#[derive(Debug, Clone)]
pub struct InputComponent {
    pub speed: f32,
    /// Whether the built-in desktop locomotion mapping may drive its target.
    pub enabled: bool,
}

impl InputComponent {
    pub fn new() -> Self {
        Self {
            speed: 0.02,
            enabled: true,
        }
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for InputComponent {
    fn default() -> Self {
        Self::new()
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

    fn init(&mut self, emit: &mut dyn crate::engine::ecs::SignalEmitter, component: ComponentId) {
        emit.push_intent_now(
            component,
            crate::engine::ecs::IntentValue::RegisterInput {
                component_id: component,
            },
        );
    }

    fn to_mms_ast(
        &self,
        _world: &crate::engine::ecs::World,
    ) -> crate::scripting::ast::ComponentExpression {
        use crate::engine::ecs::component::ce_helpers::*;
        let mut ce = ce_call("Input", "speed", vec![num(self.speed as f64)]);
        if !self.enabled {
            ce = ce.with_call("enabled", vec![b(false)]);
        }
        ce
    }
}
