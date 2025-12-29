use super::Component;
use crate::engine::ecs::ComponentId;
use crate::engine::ecs::system::SystemWorld;
use crate::engine::ecs::World;
use crate::engine::ecs::WorldContext;
use crate::engine::graphics::primitives::Transform;

#[derive(Debug, Clone, Copy)]
pub struct TransformComponent {
    /// Engine-wide transform type (also used by renderer/VisualWorld).
    pub transform: Transform,

    component: Option<ComponentId>,
}

impl TransformComponent {
    pub fn new() -> Self {
        let transform = Transform::default();
        Self {
            transform,
            component: None,
        }
    }

    fn recompute_model(&mut self) {
        self.transform.recompute_model();
    }
    
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.transform.translation = [x, y, z];
        self.recompute_model();
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.transform.scale = [x, y, z];
        self.recompute_model();
        self
    }

    pub fn with_rotation_xyzw(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.transform.rotation = [x, y, z, w];
        self.recompute_model();
        self
    }

    /// Set translation and notify `TransformSystem`.
    pub fn set_position(
        &mut self,
        ctx: &mut WorldContext,
        x: f32,
        y: f32,
        z: f32,
    ) {
        self.transform.translation = [x, y, z];
        self.recompute_model();
        let Some(cid) = self.component else { return; };
        ctx.systems
            .transform_changed(ctx.world, ctx.visuals, cid);

        // If this transform is part of an active camera (Camera2D/Camera), let CameraSystem react.
        ctx.systems
            .camera_transform_changed(ctx.world, ctx.visuals, cid);
    }

    /// Set non-uniform scale and notify `TransformSystem`.
    pub fn set_scale(
        &mut self,
        ctx: &mut WorldContext,
        x: f32,
        y: f32,
        z: f32,
    ) {
        self.transform.scale = [x, y, z];
        self.recompute_model();
        let Some(cid) = self.component else { return; };
        ctx.systems
            .transform_changed(ctx.world, ctx.visuals, cid);

        ctx.systems
            .camera_transform_changed(ctx.world, ctx.visuals, cid);
    }

    /// Set rotation from Euler angles (radians), XYZ order, and notify `TransformSystem`.
    ///
    /// API matches your sketch:
    /// `transformComponent.setRotation(PI / 2, 0, 0)`.
    pub fn set_rotation_euler(
        &mut self,
        ctx: &mut WorldContext,
        pitch_x: f32,
        yaw_y: f32,
        roll_z: f32,
    ) {
        // Minimal Euler->quat (XYZ intrinsic) implementation.
        // We'll eventually replace this with glam.
        let (sx, cx) = (0.5 * pitch_x).sin_cos();
        let (sy, cy) = (0.5 * yaw_y).sin_cos();
        let (sz, cz) = (0.5 * roll_z).sin_cos();

        // q = qx * qy * qz
        let qx = [sx, 0.0, 0.0, cx];
        let qy = [0.0, sy, 0.0, cy];
        let qz = [0.0, 0.0, sz, cz];

        fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
            let (ax, ay, az, aw) = (a[0], a[1], a[2], a[3]);
            let (bx, by, bz, bw) = (b[0], b[1], b[2], b[3]);
            [
                aw * bx + ax * bw + ay * bz - az * by,
                aw * by - ax * bz + ay * bw + az * bx,
                aw * bz + ax * by - ay * bx + az * bw,
                aw * bw - ax * bx - ay * by - az * bz,
            ]
        }

        let qxy = quat_mul(qx, qy);
        let q = quat_mul(qxy, qz);
        self.transform.rotation = q;
        self.recompute_model();

        let Some(cid) = self.component else { return; };
        ctx.systems
            .transform_changed(ctx.world, ctx.visuals, cid);

        ctx.systems
            .camera_transform_changed(ctx.world, ctx.visuals, cid);
    }
}

impl Component for TransformComponent {
    fn set_id(&mut self, component: ComponentId) {
        self.component = Some(component);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    // For now Transform doesn't need initialization.
    fn init(
        &mut self,
        _world: &mut World,
        _systems: &mut SystemWorld,
        _visuals: &mut crate::engine::graphics::VisualWorld,
        _component: ComponentId,
    ) {
    }
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self::new()
    }
}
