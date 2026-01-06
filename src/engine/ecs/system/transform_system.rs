use crate::engine::ecs::component::{Camera2DComponent, RenderableComponent, TransformComponent};
use crate::engine::ecs::ComponentId;
use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::VisualWorld;
use crate::engine::user_input::InputState;

/// System responsible for syncing `TransformComponent` changes into `VisualWorld`.
///
/// Key points:
/// - An entity can have multiple TransformComponents.
/// - A `TransformComponent` can parent other transforms to form groups.
/// - Instances in `VisualWorld` are created per `RenderableComponent` under transforms.
#[derive(Debug, Default)]
pub struct TransformSystem;

impl TransformSystem {
    pub fn new() -> Self {
        Self
    }

    fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut out = [[0.0f32; 4]; 4];
        for c in 0..4 {
            for r in 0..4 {
                out[c][r] = a[0][r] * b[c][0]
                    + a[1][r] * b[c][1]
                    + a[2][r] * b[c][2]
                    + a[3][r] * b[c][3];
            }
        }
        out
    }

    /// Compute the world-space model matrix for a component by walking up the component tree
    /// and multiplying all ancestor `TransformComponent` model matrices.
    ///
    /// Returns `None` if there are no ancestor transforms.
    pub fn world_model(world: &World, cid: ComponentId) -> Option<[[f32; 4]; 4]> {
        let mut transforms: Vec<[[f32; 4]; 4]> = Vec::new();
        let mut cur = cid;
        while let Some(parent) = world.parent_of(cur) {
            if let Some(t) = world.get_component_by_id_as::<TransformComponent>(parent) {
                transforms.push(t.transform.model);
            }
            cur = parent;
        }

        if transforms.is_empty() {
            return None;
        }

        transforms.reverse();
        let mut model = transforms[0];
        for m in transforms.into_iter().skip(1) {
            model = Self::mat4_mul(model, m);
        }
        Some(model)
    }

    /// Compute the world-space position (translation) for a component.
    pub fn world_position(world: &World, cid: ComponentId) -> Option<[f32; 3]> {
        let model = Self::world_model(world, cid)?;
        // Column-major translation lives in the last column.
        let p = model[3];
        Some([p[0], p[1], p[2]])
    }

    /// Called by TransformComponent when its values change.
    ///
    /// This updates camera translation if the transform has a Camera2D child, and updates
    /// VisualWorld instance model matrices for any `RenderableComponent` descendants.
    pub fn transform_changed(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
        camera_system: &mut crate::engine::ecs::system::CameraSystem,
        light_system: &mut crate::engine::ecs::system::LightSystem,
    ) {
        // If this transform has a Camera2D child, update camera translation.
        if let Some(camera2d_cid) = world
            .children_of(component)
            .iter()
            .copied()
            .find(|&cid| world.get_component_by_id_as::<Camera2DComponent>(cid).is_some())
        {
            camera_system.update_camera_2d_from_parent_transform(world, visuals, camera2d_cid, component);
        }

        // If any point lights live under this transform, update their world-space position.
        light_system.transform_changed(world, visuals, component);

        // Update all renderable instances in the subtree rooted at this transform.
        let mut stack = vec![component];
        while let Some(node) = stack.pop() {
            for &child in world.children_of(node) {
                stack.push(child);

                if world.get_component_by_id_as::<RenderableComponent>(child).is_some() {
                    let Some(handle) = world
                        .get_component_by_id_as::<RenderableComponent>(child)
                        .and_then(|r| r.get_handle())
                    else {
                        continue;
                    };

                    if let Some(model) = Self::world_model(world, child) {
                        visuals.update_model(handle, model);
                    }
                }
            }
        }
    }
}

impl System for TransformSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState, _dt_sec: f32) {
        // No-op. Transform updates are event-driven via `transform_changed`.
    }
}
