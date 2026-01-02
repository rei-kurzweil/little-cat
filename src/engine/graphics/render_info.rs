// Render-related helper functions and types that don't neatly fit anywhere else.
//
// Currently just owns the LC_PRINT_PIPELINE_LAYOUTS env-var gating and printing.

use crate::engine::graphics::primitives::{BufferHandle, MeshHandle};
use std::sync::Arc;

use ash::vk;

pub struct RenderInfo;

impl RenderInfo {
    pub fn maybe_print_pipeline_layouts<M: std::fmt::Debug, Me: std::fmt::Debug>(
        current_frame: usize,
        material: &M,
        mesh: &Me,
        pipeline_u64: u64,
        layout_u64: Option<u64>,
        push_constant_size_bytes: usize,
        batch_index: usize,
        batch_len: usize,
        start: u32,
        count: u32,
    ) {
        if std::env::var("LC_PRINT_PIPELINE_LAYOUTS").ok().as_deref() != Some("1") || current_frame != 0 {
            return;
        }

        println!(
            "[Renderer] pipeline/layout debug: material={:?} mesh={:?} pipeline=0x{:x} layout={}",
            material,
            mesh,
            pipeline_u64,
            layout_u64
                .map(|l| format!("0x{:x}", l))
                .unwrap_or_else(|| "<missing>".to_string()),
        );
        println!(
            "[Renderer] expected push-constant range: stage=VERTEX offset=0 size={} bytes",
            push_constant_size_bytes
        );
        println!(
            "[Renderer] batch idx {}/{} start={} count={}",
            batch_index, batch_len, start, count
        );
    }
}