use crate::engine::ecs::ComponentId;
use crate::engine::ecs::component::Component;

/// Reference to a texture image by URI.
///
/// This component is intended to be attached as a descendant of a `RenderableComponent`.
/// The URI is stored in `TextureSystem`; loading, decoding, and GPU upload happen when the
/// system sees the texture is attached to a renderable.
#[derive(Debug, Clone)]
pub struct TextureComponent {
    pub uri: String,
}

impl TextureComponent {
    pub fn new(uri: impl Into<String>) -> Self {
        Self { uri: uri.into() }
    }

    /// Construct a texture component referencing a PNG file.
    ///
    /// Currently, the engine treats `uri` as a local filesystem path (optionally prefixed
    /// with `file://`).
    pub fn from_png(uri: impl Into<String>) -> Self {
        Self::new(uri)
    }
}

impl Component for TextureComponent {
    fn name(&self) -> &'static str {
        "texture"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self, queue: &mut crate::engine::ecs::CommandQueue, component: ComponentId) {
        queue.queue_register_texture(component);
    }
}
