/// Mesh helpers / basic primitives placeholder.


/// Minimal transform (placeholder).
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // quat xyzw
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0], // identity quat
            scale: [1.0; 3],
        }
    }
}



/// Renderable component: references renderer-managed resources.
/// Vulkan-minded: mesh -> vertex/index buffers; material -> pipeline/layout + descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Renderable {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
}

impl Renderable {
    pub fn new(mesh: MeshHandle, material: MaterialHandle) -> Self {
        Self { mesh, material }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u32);

/// Vertex buffer layout description (API-agnostic placeholder).
#[derive(Debug, Clone)]
pub struct VertexLayout {
    pub stride: u32,
    pub attributes: &'static [VertexAttribute],
}

#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub location: u32,
    pub offset: u32,
    pub format: VertexFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum VertexFormat {
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
}

/// Renderer-owned mesh resource (looked up by `MeshHandle`).
#[derive(Debug, Clone, Copy)]
pub struct Mesh {
    pub vertex_buffer: BufferHandle,
    pub index_buffer: BufferHandle,
    pub index_count: u32,
    pub vertex_layout: &'static VertexLayout,
}



/// Renderer-owned resource handles (lightweight ids into renderer/asset tables).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceHandle(pub u32);

/// Renderer-owned material definition (API-agnostic placeholder).
/// For now we reference shaders by name/path; later this becomes pipeline state + descriptor layouts.
#[derive(Debug, Clone)]
pub struct Material {
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,

    // Later:
    // pub pipeline_config: PipelineConfig,
    // pub uniforms: MaterialUniforms,
}

// Optional convenience: built-in material names/paths.
impl Material {
    pub const UNLIT_FULLSCREEN: Material = Material {
        vertex_shader: "engine/graphics/shaders/vertex/fullscreen-triangle.glsl",
        fragment_shader: "engine/graphics/shaders/fragment/unlit-shader.glsl",
    };
}

impl MeshHandle {
    pub const CUBE: MeshHandle = MeshHandle(0);
    pub const TETRAHEDRON: MeshHandle = MeshHandle(1);
}

impl MaterialHandle {
    pub const UNLIT_FULLSCREEN: MaterialHandle = MaterialHandle(0);
}