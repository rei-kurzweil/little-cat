use crate::engine::ecs::component::{Component, ComponentRef};
use crate::engine::ecs::{ComponentId, IntentValue, SignalEmitter};

#[derive(Debug, Clone, PartialEq)]
pub struct SliderComponent {
    min: f32,
    max: f32,
    step: Option<f32>,
    value: f32,
    width: f32,
    disabled: bool,
    pub(crate) track: Option<ComponentRef>,
    pub(crate) thumb: Option<ComponentRef>,
    pub(crate) track_mount: Option<ComponentId>,
    pub(crate) thumb_mount: Option<ComponentId>,
}

impl SliderComponent {
    pub const DEFAULT_WIDTH: f32 = 4.0;

    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            step: None,
            value: 0.0,
            width: Self::DEFAULT_WIDTH,
            disabled: false,
            track: None,
            thumb: None,
            track_mount: None,
            thumb_mount: None,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Result<Self, String> {
        if !min.is_finite() || !max.is_finite() || min >= max {
            return Err("Slider.range requires finite min < max".into());
        }
        self.min = min;
        self.max = max;
        self.value = self.normalize(self.value);
        Ok(self)
    }

    pub fn with_step(mut self, step: f32) -> Result<Self, String> {
        if !step.is_finite() || step <= 0.0 {
            return Err("Slider.step requires a finite positive value".into());
        }
        self.step = Some(step);
        self.value = self.normalize(self.value);
        Ok(self)
    }

    pub fn with_value(mut self, value: f32) -> Result<Self, String> {
        if !value.is_finite() {
            return Err("Slider.value requires a finite value".into());
        }
        self.value = self.normalize(value);
        Ok(self)
    }

    pub fn with_width(mut self, width: f32) -> Result<Self, String> {
        if !width.is_finite() || width <= 0.0 {
            return Err("Slider.width requires a finite positive value".into());
        }
        self.width = width;
        Ok(self)
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_track(mut self, track: ComponentRef) -> Self {
        self.track = Some(track);
        self
    }

    pub fn with_thumb(mut self, thumb: ComponentRef) -> Self {
        self.thumb = Some(thumb);
        self
    }

    pub fn min(&self) -> f32 {
        self.min
    }
    pub fn max(&self) -> f32 {
        self.max
    }
    pub fn step(&self) -> Option<f32> {
        self.step
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    pub fn normalize(&self, value: f32) -> f32 {
        let value = value.clamp(self.min, self.max);
        let Some(step) = self.step else { return value };
        let snapped = self.min + ((value - self.min) / step).round() * step;
        if (value - self.max).abs() <= f32::EPSILON * self.max.abs().max(1.0) {
            self.max
        } else {
            snapped.clamp(self.min, self.max)
        }
    }

    pub(crate) fn set_value(&mut self, value: f32) -> bool {
        let next = self.normalize(value);
        if (next - self.value).abs() <= f32::EPSILON * self.value.abs().max(1.0) {
            return false;
        }
        self.value = next;
        true
    }

    pub fn fraction(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }
}

impl Default for SliderComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SliderComponent {
    fn name(&self) -> &'static str {
        "slider"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn init(&mut self, emit: &mut dyn SignalEmitter, component: ComponentId) {
        emit.push_intent_now(
            component,
            IntentValue::RegisterSlider {
                component_id: component,
            },
        );
    }
    fn to_mms_ast(
        &self,
        _world: &crate::engine::ecs::World,
    ) -> crate::scripting::ast::ComponentExpression {
        use crate::engine::ecs::component::ce_helpers::*;
        let mut expression = ce("Slider")
            .with_call("range", nums([self.min as f64, self.max as f64]))
            .with_call("value", vec![num(self.value as f64)])
            .with_call("width", vec![num(self.width as f64)]);
        if let Some(step) = self.step {
            expression = expression.with_call("step", vec![num(step as f64)]);
        }
        if self.disabled {
            expression = expression.with_call("disabled", vec![b(true)]);
        }
        if let Some(reference) = &self.track {
            let value = match reference {
                ComponentRef::Guid(guid) => s(&format!("@uuid:{guid}")),
                ComponentRef::Query(query) => s(query),
            };
            expression = expression.with_call("track", vec![value]);
        }
        if let Some(reference) = &self.thumb {
            let value = match reference {
                ComponentRef::Guid(guid) => s(&format!("@uuid:{guid}")),
                ComponentRef::Query(query) => s(query),
            };
            expression = expression.with_call("thumb", vec![value]);
        }
        expression
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_and_snaps_from_min_with_reachable_endpoints() {
        let slider = SliderComponent::new()
            .range(1.0, 2.0)
            .unwrap()
            .with_step(0.3)
            .unwrap();
        assert!((slider.normalize(1.44) - 1.3).abs() < 1e-6);
        assert_eq!(slider.normalize(-5.0), 1.0);
        assert_eq!(slider.normalize(2.0), 2.0);
    }

    #[test]
    fn rejects_invalid_configuration() {
        assert!(SliderComponent::new().range(1.0, 1.0).is_err());
        assert!(SliderComponent::new().with_step(0.0).is_err());
        assert!(SliderComponent::new().with_width(f32::NAN).is_err());
    }
}
