
use crate::engine::ecs::system::System;
use crate::engine::ecs::ComponentId;
use crate::engine::ecs::{World};
use crate::engine::graphics::VisualWorld;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CameraHandle(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

#[derive(Debug, Clone, Copy)]
enum AnyCamera {
    Camera3D(Camera),
}

impl Camera {
    pub fn identity() -> Self {
        Self {
            view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Right-handed perspective projection matrix.
    ///
    /// Assumptions:
    /// - Column-major mat4 (matches how we pack instance matrices / GLSL default).
    /// - NDC depth range is Vulkan-style: z in [0, 1].
    pub fn perspective_rh_zo(fov_y_radians: f32, aspect: f32, z_near: f32, z_far: f32) -> [[f32; 4]; 4] {
        // Based on the standard RH, zero-to-one depth projection.
        // Maps camera forward -Z.
        let f = 1.0 / (0.5 * fov_y_radians).tan();
        let nf = 1.0 / (z_near - z_far);

        // Column-major:
        // [ f/aspect, 0,  0,                      0 ]
        // [ 0,        f,  0,                      0 ]
        // [ 0,        0,  z_far*nf,               -1 ]
        // [ 0,        0,  z_near*z_far*nf,         0 ]
        [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, z_far * nf, -1.0],
            [0.0, 0.0, (z_near * z_far) * nf, 0.0],
        ]
    }
}

#[derive(Debug, Default)]
pub struct CameraSystem {
    next_handle: u32,
    cameras: Vec<(CameraHandle, AnyCamera)>,
    pub active_camera: Option<CameraHandle>,
}

impl CameraSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a camera derived from the component tree.
    ///
    /// The newest registered camera becomes active.
    pub fn register_camera(
        &mut self,
        _world: &mut World,
        visuals: &mut VisualWorld,
        _component: ComponentId,
    ) -> CameraHandle {
        // NOTE: Debug step: force BOTH view and projection to identity to fully isolate
        // whether the camera path (push constants, shader bindings, etc.) is the cause.
        // (So we also intentionally ignore any camera transform for now.)
        let cam = Camera::identity();

        let h = CameraHandle(self.next_handle);
        self.next_handle = self.next_handle.wrapping_add(1);

        self.cameras.push((h, AnyCamera::Camera3D(cam)));

        // Newest becomes active.
        self.active_camera = Some(h);
        visuals.set_camera(cam.view, cam.proj);

        h
    }

    pub fn set_active_camera(&mut self, visuals: &mut VisualWorld, h: CameraHandle) {
        if self.active_camera == Some(h) {
            return;
        }

        if let Some((_, cam)) = self.cameras.iter().find(|(ch, _)| *ch == h) {
            self.active_camera = Some(h);
            match *cam {
                AnyCamera::Camera3D(cam3d) => {
                    visuals.set_camera(cam3d.view, cam3d.proj);
                }
            }
        }
    }

    pub fn active_camera_matrices(&self) -> Option<([[f32; 4]; 4], [[f32; 4]; 4])> {
        let h = self.active_camera?;
        let (_, cam) = self.cameras.iter().find(|(ch, _)| *ch == h)?;
        match *cam {
            AnyCamera::Camera3D(cam3d) => Some((cam3d.view, cam3d.proj)),
        }
    }
}

/// Invert a TRS matrix assuming it's only translation + scale (no rotation/shear).
///
/// This matches how the demo currently uses `TransformComponent` (position + scale only).
/// If/when we add rotations, we'll want a full mat4 inverse or a quat-based view build.
fn invert_rigid_transform(m: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    // Column-major, with translation in column 3 (index 3).
    // Our Transform builder also stores translation in m[3][0..2] (4th column).
    let sx = m[0][0];
    let sy = m[1][1];
    let sz = m[2][2];

    // Protect against divide-by-zero.
    let inv_sx = if sx.abs() > 1e-8 { 1.0 / sx } else { 1.0 };
    let inv_sy = if sy.abs() > 1e-8 { 1.0 / sy } else { 1.0 };
    let inv_sz = if sz.abs() > 1e-8 { 1.0 / sz } else { 1.0 };

    let tx = m[3][0];
    let ty = m[3][1];
    let tz = m[3][2];

    // Inverse of S then T: inv(M) = inv(S) * inv(T)
    // For column-major with translation in last column: inv translation becomes -(invS * t).
    let itx = -(tx * inv_sx);
    let ity = -(ty * inv_sy);
    let itz = -(tz * inv_sz);

    [
        [inv_sx, 0.0, 0.0, 0.0],
        [0.0, inv_sy, 0.0, 0.0],
        [0.0, 0.0, inv_sz, 0.0],
        [itx, ity, itz, 1.0],
    ]
}

impl System for CameraSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &crate::engine::user_input::InputState) {
        // No-op for now.
    }
}
