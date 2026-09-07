use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::engine::ecs::component::{
    ColorComponent, ComponentRef, DragContinuationPolicy, DragMappingPolicy, QueryRootMode,
    RaycastableComponent, RenderableComponent, SerializeComponent, SliderComponent,
    TransformComponent, resolve_component_ref,
};
use crate::engine::ecs::system::TransformSystem;
use crate::engine::ecs::{
    ComponentId, EventSignal, IntentValue, PointerActivationSource, RxWorld, SignalEmitter,
    SignalKind, World,
};

const TRACK_MOUNT: &str = "slider_track_mount";
const THUMB_MOUNT: &str = "slider_thumb_mount";

type GestureKey = (u8, ComponentId, ComponentId);

#[derive(Debug, Clone, Copy)]
struct ActiveSlider {
    slider: ComponentId,
    local_grab_offset: f32,
}

#[derive(Debug, Default)]
pub struct SliderSystem {
    handlers_installed: bool,
    active: Arc<Mutex<HashMap<GestureKey, ActiveSlider>>>,
    focused: Arc<Mutex<Option<ComponentId>>>,
}

impl SliderSystem {
    pub fn install_handlers(&mut self, rx: &mut RxWorld) {
        if self.handlers_installed {
            return;
        }
        self.handlers_installed = true;

        let active = Arc::clone(&self.active);
        let focused = Arc::clone(&self.focused);
        rx.add_global_handler_closure(SignalKind::DragStart, move |world, emit, env| {
            let Some(EventSignal::DragStart {
                activation_source,
                raycaster,
                renderable,
                hit_point,
                ..
            }) = env.event.as_ref()
            else {
                return;
            };
            if *activation_source != PointerActivationSource::Trigger {
                return;
            }
            let Some(slider) = nearest_slider(world, *renderable) else {
                return;
            };
            if world
                .get_component_by_id_as::<SliderComponent>(slider)
                .is_some_and(SliderComponent::disabled)
            {
                return;
            }
            *focused.lock().expect("slider focus lock") = Some(slider);
            let Some(local_x) = local_x(world, slider, *hit_point) else {
                return;
            };
            let thumb_hit = world
                .get_component_by_id_as::<SliderComponent>(slider)
                .and_then(|s| s.thumb_mount)
                .is_some_and(|thumb| {
                    thumb == *renderable || world.is_ancestor_of(thumb, *renderable)
                });
            let thumb_x = slider_thumb_x(world, slider).unwrap_or(local_x);
            let grab_offset = if thumb_hit { thumb_x - local_x } else { 0.0 };
            active.lock().expect("slider gesture lock").insert(
                gesture_key(*activation_source, *raycaster, *renderable),
                ActiveSlider {
                    slider,
                    local_grab_offset: grab_offset,
                },
            );
            apply_pointer_value(world, emit, slider, local_x + grab_offset);
        });

        let active = Arc::clone(&self.active);
        rx.add_global_handler_closure(SignalKind::DragMove, move |world, emit, env| {
            let Some(EventSignal::DragMove {
                activation_source,
                raycaster,
                renderable,
                hit_point,
                ..
            }) = env.event.as_ref()
            else {
                return;
            };
            let Some(active_slider) = active
                .lock()
                .expect("slider gesture lock")
                .get(&gesture_key(*activation_source, *raycaster, *renderable))
                .copied()
            else {
                return;
            };
            let Some(local_x) = local_x(world, active_slider.slider, *hit_point) else {
                return;
            };
            apply_pointer_value(
                world,
                emit,
                active_slider.slider,
                local_x + active_slider.local_grab_offset,
            );
        });

        let active = Arc::clone(&self.active);
        rx.add_global_handler_closure(SignalKind::DragEnd, move |world, emit, env| {
            let Some(EventSignal::DragEnd {
                activation_source,
                raycaster,
                renderable,
                ..
            }) = env.event.as_ref()
            else {
                return;
            };
            let Some(active_slider) = active
                .lock()
                .expect("slider gesture lock")
                .remove(&gesture_key(*activation_source, *raycaster, *renderable))
            else {
                return;
            };
            if let Some(value) = world
                .get_component_by_id_as::<SliderComponent>(active_slider.slider)
                .map(SliderComponent::value)
            {
                emit.push_event(
                    active_slider.slider,
                    EventSignal::SliderCommitted {
                        slider: active_slider.slider,
                        value,
                    },
                );
            }
        });

        let focused_click = Arc::clone(&self.focused);
        rx.add_global_handler_closure(SignalKind::Click, move |world, _emit, env| {
            let renderable = match env.event.as_ref() {
                Some(EventSignal::Click { renderable, .. }) => *renderable,
                _ => return,
            };
            *focused_click.lock().expect("slider focus lock") = nearest_slider(world, renderable);
        });

        for kind in [SignalKind::KeyDown, SignalKind::KeyPress] {
            let focused = Arc::clone(&self.focused);
            rx.add_global_handler_closure(kind, move |world, emit, env| {
                let event = match env.event.as_ref() {
                    Some(EventSignal::KeyDown(event)) | Some(EventSignal::KeyPress(event)) => event,
                    _ => return,
                };
                let Some(slider) = *focused.lock().expect("slider focus lock") else {
                    return;
                };
                let Some(component) = world.get_component_by_id_as::<SliderComponent>(slider)
                else {
                    return;
                };
                if component.disabled() {
                    return;
                }
                let increment = component
                    .step()
                    .unwrap_or((component.max() - component.min()) * 0.01);
                let next = match event.key.as_str() {
                    "ArrowLeft" | "ArrowDown" => component.value() - increment,
                    "ArrowRight" | "ArrowUp" => component.value() + increment,
                    "Home" => component.min(),
                    "End" => component.max(),
                    _ => return,
                };
                apply_slider_set(world, emit, slider, next, true);
            });
        }
    }

    pub fn register(
        &mut self,
        world: &mut World,
        slider: ComponentId,
        emit: &mut dyn SignalEmitter,
    ) {
        let Some(config) = world
            .get_component_by_id_as::<SliderComponent>(slider)
            .cloned()
        else {
            return;
        };
        let track_mount = ensure_mount(world, slider, TRACK_MOUNT, 0.0);
        let thumb_x = (config.fraction() - 0.5) * config.width();
        let thumb_mount = ensure_mount(world, slider, THUMB_MOUNT, thumb_x);

        if let Some(component) = world.get_component_by_id_as_mut::<SliderComponent>(slider) {
            component.track_mount = Some(track_mount);
            component.thumb_mount = Some(thumb_mount);
        }

        mount_visual(
            world,
            emit,
            slider,
            track_mount,
            config.track.as_ref(),
            true,
            config.width(),
        );
        mount_visual(
            world,
            emit,
            slider,
            thumb_mount,
            config.thumb.as_ref(),
            false,
            config.width(),
        );
    }
}

pub fn apply_slider_set(
    world: &mut World,
    emit: &mut dyn SignalEmitter,
    slider: ComponentId,
    value: f32,
    emit_changed: bool,
) {
    if !value.is_finite() {
        return;
    }
    let (changed, effective, thumb_mount, thumb_x) = {
        let Some(component) = world.get_component_by_id_as_mut::<SliderComponent>(slider) else {
            return;
        };
        let changed = component.set_value(value);
        (
            changed,
            component.value(),
            component.thumb_mount,
            (component.fraction() - 0.5) * component.width(),
        )
    };
    if let Some(thumb) = thumb_mount {
        if let Some(transform) = world.get_component_by_id_as::<TransformComponent>(thumb) {
            emit.push_intent_now(
                thumb,
                IntentValue::UpdateTransform {
                    component_id: thumb,
                    translation: [thumb_x, 0.0, 0.02],
                    rotation_quat_xyzw: transform.transform.rotation,
                    scale: transform.transform.scale,
                },
            );
        }
    }
    if changed && emit_changed {
        emit.push_event(
            slider,
            EventSignal::SliderChanged {
                slider,
                value: effective,
            },
        );
    }
}

fn apply_pointer_value(
    world: &mut World,
    emit: &mut dyn SignalEmitter,
    slider: ComponentId,
    local_x: f32,
) {
    let Some(component) = world.get_component_by_id_as::<SliderComponent>(slider) else {
        return;
    };
    let fraction = (local_x / component.width() + 0.5).clamp(0.0, 1.0);
    let value = component.min() + fraction * (component.max() - component.min());
    apply_slider_set(world, emit, slider, value, true);
}

fn local_x(world: &World, slider: ComponentId, point: [f32; 3]) -> Option<f32> {
    let model = TransformSystem::world_model(world, slider)?;
    let inverse = crate::utils::math::mat4_inverse(model)?;
    let p = crate::utils::math::mat4_mul_vec4(inverse, [point[0], point[1], point[2], 1.0]);
    Some(p[0])
}

fn slider_thumb_x(world: &World, slider: ComponentId) -> Option<f32> {
    let mount = world
        .get_component_by_id_as::<SliderComponent>(slider)?
        .thumb_mount?;
    Some(
        world
            .get_component_by_id_as::<TransformComponent>(mount)?
            .transform
            .translation[0],
    )
}

fn nearest_slider(world: &World, start: ComponentId) -> Option<ComponentId> {
    let mut current = Some(start);
    while let Some(node) = current {
        if world
            .get_component_by_id_as::<SliderComponent>(node)
            .is_some()
        {
            return Some(node);
        }
        if let Some(sidecar) = world.children_of(node).iter().copied().find(|id| {
            world
                .get_component_by_id_as::<SliderComponent>(*id)
                .is_some()
        }) {
            return Some(sidecar);
        }
        current = world.parent_of(node);
    }
    None
}

fn ensure_mount(world: &mut World, slider: ComponentId, label: &str, x: f32) -> ComponentId {
    if let Some(existing) = world
        .children_of(slider)
        .iter()
        .copied()
        .find(|id| world.component_label(*id) == Some(label))
    {
        return existing;
    }
    let mount = world.add_component_boxed_named(
        label,
        Box::new(TransformComponent::new().with_position(x, 0.0, 0.02)),
    );
    world
        .add_child(slider, mount)
        .expect("fresh slider mount attaches");
    let serialize = world.add_component(SerializeComponent::off());
    world.add_child(mount, serialize).ok();
    mount
}

fn mount_visual(
    world: &mut World,
    emit: &mut dyn SignalEmitter,
    slider: ComponentId,
    mount: ComponentId,
    authored: Option<&ComponentRef>,
    track: bool,
    width: f32,
) {
    if world.children_of(mount).iter().any(|child| {
        world
            .get_component_by_id_as::<SerializeComponent>(*child)
            .is_none()
    }) {
        return;
    }
    if let Some(root) = authored.and_then(|reference| {
        resolve_component_ref(world, reference, Some(slider), QueryRootMode::SelfSubtree)
    }) {
        if world.add_child(mount, root).is_ok() {
            let has_override = world.children_of(root).iter().any(|id| {
                world
                    .get_component_by_id_as::<SerializeComponent>(*id)
                    .is_some()
            });
            if !has_override {
                let serialize = world.add_component(SerializeComponent::on());
                world.add_child(root, serialize).ok();
            }
            ensure_subtree_raycastable(world, emit, root);
            return;
        }
    }
    let scale = if track {
        [width * 0.5, 0.07, 0.04]
    } else {
        [0.14, 0.22, 0.08]
    };
    let color = if track {
        [0.24, 0.27, 0.34, 1.0]
    } else {
        [0.95, 0.72, 0.22, 1.0]
    };
    let visual =
        world.add_component(TransformComponent::new().with_scale(scale[0], scale[1], scale[2]));
    let renderable = world.add_component(RenderableComponent::cube());
    let tint = world.add_component(ColorComponent { rgba: color });
    let mut hit = RaycastableComponent::enabled();
    hit.drag_continuation = DragContinuationPolicy::Captured;
    hit.drag_mapping = DragMappingPolicy::StartRayPlane;
    let raycastable = world.add_component(hit);
    let serialize = world.add_component(SerializeComponent::off());
    world.add_child(mount, visual).ok();
    world.add_child(visual, renderable).ok();
    world.add_child(renderable, tint).ok();
    world.add_child(renderable, raycastable).ok();
    world.add_child(visual, serialize).ok();
    world.init_component_tree(visual, emit);
}

fn ensure_subtree_raycastable(world: &mut World, emit: &mut dyn SignalEmitter, root: ComponentId) {
    let mut stack = vec![root];
    let mut renderables = Vec::new();
    while let Some(node) = stack.pop() {
        if world
            .get_component_by_id_as::<RenderableComponent>(node)
            .is_some()
        {
            renderables.push(node);
        }
        stack.extend(world.children_of(node).iter().copied());
    }
    for renderable in renderables {
        let existing = world.children_of(renderable).iter().copied().find(|id| {
            world.get_component_by_id_as::<RaycastableComponent>(*id).is_some()
        });
        if let Some(existing) = existing {
            if let Some(hit) = world.get_component_by_id_as_mut::<RaycastableComponent>(existing) {
                hit.enable = true;
                hit.drag_continuation = DragContinuationPolicy::Captured;
                hit.drag_mapping = DragMappingPolicy::StartRayPlane;
            }
        } else {
            let mut hit = RaycastableComponent::enabled();
            hit.drag_continuation = DragContinuationPolicy::Captured;
            hit.drag_mapping = DragMappingPolicy::StartRayPlane;
            let id = world.add_component(hit);
            world.add_child(renderable, id).ok();
            world.init_component_tree(id, emit);
        }
    }
}

fn gesture_key(
    source: PointerActivationSource,
    raycaster: ComponentId,
    renderable: ComponentId,
) -> GestureKey {
    (source as u8, raycaster, renderable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Events {
        events: Vec<EventSignal>,
        intents: Vec<crate::engine::ecs::IntentSignal>,
    }
    impl SignalEmitter for Events {
        fn push_event(&mut self, _: ComponentId, event: EventSignal) {
            self.events.push(event);
        }
        fn push_intent(&mut self, _: ComponentId, intent: crate::engine::ecs::IntentSignal) {
            self.intents.push(intent);
        }
    }

    #[test]
    fn silent_set_normalizes_state_and_requests_thumb_motion_without_an_event() {
        let mut world = World::default();
        let slider = world.add_component(SliderComponent::new().with_value(0.25).unwrap());
        let thumb = world.add_component(TransformComponent::new());
        world
            .get_component_by_id_as_mut::<SliderComponent>(slider)
            .unwrap()
            .thumb_mount = Some(thumb);
        let mut events = Events::default();
        apply_slider_set(&mut world, &mut events, slider, 0.75, false);
        assert_eq!(
            world
                .get_component_by_id_as::<SliderComponent>(slider)
                .unwrap()
                .value(),
            0.75
        );
        assert!(events.events.is_empty());
        assert!(events.intents.iter().any(|intent| matches!(
            &intent.value,
            IntentValue::UpdateTransform { component_id, translation, .. }
                if *component_id == thumb && (translation[0] - 1.0).abs() < 1e-6
        )));
    }

    #[test]
    fn register_creates_stable_mounts_and_reparents_authored_visual() {
        let mut world = World::default();
        let authored_track = world.add_component(TransformComponent::new());
        let track_guid = world.get_component_record(authored_track).unwrap().guid;
        let slider = world.add_component(
            SliderComponent::new()
                .with_value(0.5)
                .unwrap()
                .with_track(ComponentRef::Guid(track_guid)),
        );
        let mut events = Events::default();
        let mut system = SliderSystem::default();
        system.register(&mut world, slider, &mut events);

        let component = world
            .get_component_by_id_as::<SliderComponent>(slider)
            .unwrap();
        let track_mount = component.track_mount.expect("track mount");
        let thumb_mount = component.thumb_mount.expect("thumb mount");
        assert_eq!(world.component_label(track_mount), Some(TRACK_MOUNT));
        assert_eq!(world.component_label(thumb_mount), Some(THUMB_MOUNT));
        assert_eq!(world.parent_of(authored_track), Some(track_mount));
        assert!(world.children_of(thumb_mount).iter().any(|child| {
            world
                .get_component_by_id_as::<TransformComponent>(*child)
                .is_some()
                && *child != authored_track
        }));
    }
}
