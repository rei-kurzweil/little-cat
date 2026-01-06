use std::collections::HashMap;

use crate::engine::graphics::MeshUploader;
use crate::engine::graphics::mesh::CpuMesh;
use crate::engine::graphics::primitives::{CpuMeshHandle, MeshHandle};

/// Renderer-side asset registry used by ECS systems.
///
/// Design:
/// - ECS and gameplay code refer to geometry by `CpuMeshHandle` (CPU asset identity).
/// - The renderer owns GPU resources and returns `MeshHandle`.
/// - `RenderAssets` bridges the two and caches uploads.
#[derive(Debug, Default)]
pub struct RenderAssets {
    cpu_meshes: Vec<CpuMesh>,
    gpu_meshes: HashMap<CpuMeshHandle, MeshHandle>,
}

impl RenderAssets {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register CPU mesh data and get a stable CPU-side handle.
    ///
    /// If callers want reuse, they should keep and share this handle.
    pub fn register_mesh(&mut self, mesh: CpuMesh) -> CpuMeshHandle {
        let h = CpuMeshHandle(self.cpu_meshes.len() as u32);
        self.cpu_meshes.push(mesh);
        h
    }

    pub fn cpu_mesh(&self, h: CpuMeshHandle) -> Option<&CpuMesh> {
        self.cpu_meshes.get(h.0 as usize)
    }

    /// Get (or upload) a mesh into the renderer and return a renderer-owned `MeshHandle`.
    pub fn gpu_mesh_handle(
        &mut self,
        uploader: &mut dyn MeshUploader,
        cpu_mesh: CpuMeshHandle,
    ) -> Result<MeshHandle, Box<dyn std::error::Error>> {
        if let Some(h) = self.gpu_meshes.get(&cpu_mesh).copied() {
            return Ok(h);
        }

        let mesh = self
            .cpu_mesh(cpu_mesh)
            .ok_or("RenderAssets: invalid CpuMeshHandle")?;
        let h = uploader.upload_mesh(mesh)?;
        self.gpu_meshes.insert(cpu_mesh, h);
        Ok(h)
    }
}
