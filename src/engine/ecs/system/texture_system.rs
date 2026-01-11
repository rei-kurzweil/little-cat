use crate::engine::ecs::component::{CatEngineTextureFormat, RenderableComponent, TextureComponent};
use crate::engine::ecs::{ComponentId, World};
use crate::engine::graphics::{TextureHandle, TextureUploader, VisualWorld};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct TextureRecord {
    uri: String,
    format: CatEngineTextureFormat,
    gpu: Option<TextureHandle>,
}

#[derive(Debug, Default)]
pub struct TextureSystem {
    textures: HashMap<ComponentId, TextureRecord>,
    uri_cache: HashMap<String, TextureHandle>,
    /// RenderableComponent cid -> TextureComponent cid
    pending_attach: HashMap<ComponentId, ComponentId>,
}

impl TextureSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_texture(
        &mut self,
        world: &mut World,
        _visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        let Some(tex_comp) = world.get_component_by_id_as::<TextureComponent>(component) else {
            return;
        };

        self.textures
            .entry(component)
            .or_insert_with(|| TextureRecord {
                uri: tex_comp.uri.clone(),
                format: tex_comp.format,
                gpu: None,
            });

        // If this texture is attached under a renderable, remember that relationship.
        let mut cur = component;
        while let Some(parent) = world.parent_of(cur) {
            if world
                .get_component_by_id_as::<RenderableComponent>(parent)
                .is_some()
            {
                self.pending_attach.insert(parent, component);
                break;
            }
            cur = parent;
        }
    }

    /// Decode+upload any textures that are now attachable to renderables.
    ///
    /// Must run after renderables are flushed into `VisualWorld` so we can update instance handles.
    pub fn flush_pending(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        uploader: &mut dyn TextureUploader,
    ) {
        let pairs: Vec<(ComponentId, ComponentId)> =
            self.pending_attach.iter().map(|(&r, &t)| (r, t)).collect();

        for (renderable_cid, texture_cid) in pairs {
            let Some(renderable_comp) =
                world.get_component_by_id_as::<RenderableComponent>(renderable_cid)
            else {
                let _ = self.pending_attach.remove(&renderable_cid);
                continue;
            };

            let Some(instance_handle) = renderable_comp.get_handle() else {
                // Renderable not in VisualWorld yet.
                continue;
            };

            let Some(record) = self.textures.get_mut(&texture_cid) else {
                let _ = self.pending_attach.remove(&renderable_cid);
                continue;
            };

            if let Some(cached) = self.uri_cache.get(&record.uri).copied() {
                record.gpu = Some(cached);
            }

            let tex_handle = match record.gpu {
                Some(h) => h,
                None => {
                    let uri = record.uri.as_str();
                    let raw_path_str = uri.strip_prefix("file://").unwrap_or(uri);
                    let raw_path = Path::new(raw_path_str);

                    let mut tried: Vec<PathBuf> = Vec::new();
                    let resolved_path: Option<PathBuf> = if raw_path.is_absolute() {
                        tried.push(raw_path.to_path_buf());
                        if raw_path.exists() {
                            Some(raw_path.to_path_buf())
                        } else {
                            None
                        }
                    } else {
                        // 1) Current working directory
                        if let Ok(cwd) = std::env::current_dir() {
                            let p = cwd.join(raw_path);
                            tried.push(p.clone());
                            if p.exists() {
                                Some(p)
                            } else {
                                // 2) Crate root (works even if CWD is target/...)
                                let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                                let p2 = manifest_dir.join(raw_path);
                                tried.push(p2.clone());
                                if p2.exists() { Some(p2) } else { None }
                            }
                        } else {
                            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                            let p2 = manifest_dir.join(raw_path);
                            tried.push(p2.clone());
                            if p2.exists() { Some(p2) } else { None }
                        }
                    };

                    let Some(path) = resolved_path else {
                        let cwd = std::env::current_dir()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| "<unknown>".to_string());
                        println!("[TextureSystem] read failed for '{uri}'");
                        println!("[TextureSystem]   cwd = {cwd}");
                        for p in tried {
                            println!("[TextureSystem]   tried: {}", p.display());
                        }
                        let _ = self.pending_attach.remove(&renderable_cid);
                        continue;
                    };

                    let bytes = match std::fs::read(&path) {
                        Ok(b) => b,
                        Err(e) => {
                            let cwd = std::env::current_dir()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|_| "<unknown>".to_string());
                            println!("[TextureSystem] read failed for '{uri}': {e}");
                            println!("[TextureSystem]   cwd = {cwd}");
                            println!("[TextureSystem]   resolved: {}", path.display());
                            let _ = self.pending_attach.remove(&renderable_cid);
                            continue;
                        }
                    };

                    let handle = match record.format {
                        CatEngineTextureFormat::DdsBc7 => {
                        match decode_dds_bc7(&bytes) {
                            Ok(decoded) => match uploader.upload_texture_bc7(
                                &decoded.bc7_blocks,
                                decoded.width,
                                decoded.height,
                                decoded.srgb,
                            ) {
                                Ok(h) => h,
                                Err(e) => {
                                    println!("[TextureSystem] BC7 upload failed for '{uri}': {:?}", e);
                                    let _ = self.pending_attach.remove(&renderable_cid);
                                    continue;
                                }
                            },
                            Err(e) => {
                                println!("[TextureSystem] DDS/BC7 decode failed for '{uri}': {e}");
                                let _ = self.pending_attach.remove(&renderable_cid);
                                continue;
                            }
                        }
                        }
                        CatEngineTextureFormat::Rgba8 => {
                        let dyn_img = match image::load_from_memory(&bytes) {
                            Ok(i) => i,
                            Err(e) => {
                                println!("[TextureSystem] decode failed for '{uri}': {:?}", e);
                                let _ = self.pending_attach.remove(&renderable_cid);
                                continue;
                            }
                        };

                        let rgba = dyn_img.to_rgba8();
                        let (w, h) = rgba.dimensions();

                        match uploader.upload_texture_rgba8(rgba.as_raw(), w, h) {
                            Ok(h) => h,
                            Err(e) => {
                                println!("[TextureSystem] upload failed for '{uri}': {:?}", e);
                                let _ = self.pending_attach.remove(&renderable_cid);
                                continue;
                            }
                        }
                        }
                    };

                    record.gpu = Some(handle);
                    self.uri_cache.insert(record.uri.clone(), handle);
                    handle
                }
            };

            let _ = visuals.update_texture(instance_handle, Some(tex_handle));
            let _ = self.pending_attach.remove(&renderable_cid);
        }
    }
}

struct Bc7Decoded {
    width: u32,
    height: u32,
    srgb: bool,
    bc7_blocks: Vec<u8>,
}

fn decode_dds_bc7(bytes: &[u8]) -> Result<Bc7Decoded, String> {
    let mut cursor = Cursor::new(bytes);
    let dds = ddsfile::Dds::read(&mut cursor).map_err(|e| format!("{e:?}"))?;

    let width = dds.get_width();
    let height = dds.get_height();
    if width == 0 || height == 0 {
        return Err("DDS has zero size".to_string());
    }

    let dxgi = dds
        .get_dxgi_format()
        .ok_or_else(|| "DDS missing DXGI format (need BC7 in DX10 header)".to_string())?;

    let srgb = match dxgi {
        ddsfile::DxgiFormat::BC7_UNorm => false,
        ddsfile::DxgiFormat::BC7_UNorm_sRGB => true,
        other => {
            return Err(format!("DDS is not BC7 (got {other:?})"));
        }
    };

    let data: &[u8] = dds.data.as_ref();
    if data.is_empty() {
        return Err("DDS contains no data".to_string());
    }

    // We only use the top mip for now.
    let blocks_w = (width + 3) / 4;
    let blocks_h = (height + 3) / 4;
    let expected_len = blocks_w as usize * blocks_h as usize * 16;
    if data.len() < expected_len {
        return Err(format!(
            "DDS data too small for BC7 level 0: got={}, need={}",
            data.len(),
            expected_len
        ));
    }

    Ok(Bc7Decoded {
        width,
        height,
        srgb,
        bc7_blocks: data[..expected_len].to_vec(),
    })
}
