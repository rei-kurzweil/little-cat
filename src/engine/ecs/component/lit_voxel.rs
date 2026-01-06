use super::Component;
use crate::engine::ecs::ComponentId;

/// Per-instance voxel lighting/shading metadata.
///
/// Intended usage:
/// - A CPU system computes `shade_level` / `emissive` for many voxels.
/// - The renderer consumes a GPU buffer (SSBO) indexed by `gl_InstanceIndex`.
#[derive(Debug, Clone, Copy)]
pub struct LitVoxelComponent {
    /// Quantized shade level (0 = fully lit).
    pub shade_level: u8,

    /// Secondary effect; if true, voxel emits light / glows.
    pub emissive: bool,

    component: Option<ComponentId>,
}

impl LitVoxelComponent {
    pub fn new() -> Self {
        Self {
            shade_level: 0,
            emissive: false,
            component: None,
        }
    }

    pub fn with_shade_level(mut self, shade_level: u8) -> Self {
        self.shade_level = shade_level;
        self
    }

    pub fn with_emissive(mut self, emissive: bool) -> Self {
        self.emissive = emissive;
        self
    }

    pub fn id(&self) -> Option<ComponentId> {
        self.component
    }
}

impl Default for LitVoxelComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for LitVoxelComponent {
    fn name(&self) -> &'static str {
        "lit_voxel"
    }

    fn set_id(&mut self, component: ComponentId) {
        self.component = Some(component);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
