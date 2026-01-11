use crate::engine::ecs::ComponentId;
use crate::engine::ecs::component::Component;
use std::path::Path;

/// Runtime texture source/encoding understood by the engine.
///
/// This is intentionally *not* serialized; it is derived from `uri` when the component is
/// created/decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatEngineTextureFormat {
    /// Any image format decodable by the `image` crate; uploaded as RGBA8.
    Rgba8,
    /// DDS container containing BC7 blocks (UNorm or UNorm_sRGB).
    DdsBc7,
}

impl CatEngineTextureFormat {
    pub fn from_uri(uri: &str) -> Self {
        // We currently treat `uri` as a filesystem path (optionally prefixed with `file://`).
        let raw_path_str = uri.strip_prefix("file://").unwrap_or(uri);
        let ext = Path::new(raw_path_str)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if ext.eq_ignore_ascii_case("dds") {
            CatEngineTextureFormat::DdsBc7
        } else {
            CatEngineTextureFormat::Rgba8
        }
    }
}

/// Reference to a texture image by URI.
///
/// This component is intended to be attached as a descendant of a `RenderableComponent`.
/// The URI is stored in `TextureSystem`; loading, decoding, and GPU upload happen when the
/// system sees the texture is attached to a renderable.
#[derive(Debug, Clone)]
pub struct TextureComponent {
    pub uri: String,
    pub format: CatEngineTextureFormat,
}

impl TextureComponent {
    pub fn new(uri: impl Into<String>) -> Self {
        let uri = uri.into();
        let format = CatEngineTextureFormat::from_uri(&uri);
        Self { uri, format }
    }

    /// Construct a texture component referencing a PNG file.
    ///
    /// Currently, the engine treats `uri` as a local filesystem path (optionally prefixed
    /// with `file://`).
    pub fn from_png(uri: impl Into<String>) -> Self {
        let mut c = Self::new(uri);
        c.format = CatEngineTextureFormat::Rgba8;
        c
    }

    /// Construct a texture component referencing a DDS file containing BC7 blocks.
    pub fn from_dds(uri: impl Into<String>) -> Self {
        let mut c = Self::new(uri);
        c.format = CatEngineTextureFormat::DdsBc7;
        c
    }

    pub fn refresh_format_from_uri(&mut self) {
        self.format = CatEngineTextureFormat::from_uri(&self.uri);
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

    fn encode(&self) -> std::collections::HashMap<String, serde_json::Value> {
        let mut map = std::collections::HashMap::new();
        map.insert("uri".to_string(), serde_json::json!(self.uri));
        map
    }

    fn decode(
        &mut self,
        data: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        if let Some(uri) = data.get("uri") {
            self.uri = serde_json::from_value(uri.clone())
                .map_err(|e| format!("Failed to decode uri: {}", e))?;
        }
        self.refresh_format_from_uri();
        Ok(())
    }
}
