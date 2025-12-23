/// Camera placeholder (flat + VR-friendly later).
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub fov_y_radians: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov_y_radians: 60.0_f32.to_radians(),
        }
    }
}
