use crate::engine::ecs::entity::ComponentId;
use crate::engine::ecs::World;
use crate::engine::graphics::{MaterialHandle, VisualWorld};

/// Lightweight, stdout-based renderer + ECS diagnostics.
///
/// Intended for bring-up: call from the render loop to see whether ECS produced
/// any renderables, whether `VisualWorld` built draw batches, and whether instance
/// data is being packed/uploaded.
#[derive(Debug, Default, Clone)]
pub struct RenderingInspector {
    /// If true, print every frame. If false, print only when the summary changes.
    pub verbose_every_frame: bool,

    last_signature: Option<Signature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Signature {
    instances_len: usize,
    draw_order_len: usize,
    draw_batches_len: usize,
    cached_instance_floats: usize,
    instance_buffer_capacity: usize,
}

impl RenderingInspector {
    pub fn new() -> Self {
        Self::default()
    }

    fn should_print(&mut self, sig: Signature) -> bool {
        if self.verbose_every_frame {
            self.last_signature = Some(sig);
            return true;
        }

        match self.last_signature {
            Some(old) if old == sig => false,
            _ => {
                self.last_signature = Some(sig);
                true
            }
        }
    }

    /// Print a high-level summary of ECS + VisualWorld.
    pub fn print_world_and_visuals(
        &mut self,
        stage: &str,
        world: &World,
        visual_world: &VisualWorld,
        cached_instance_floats: usize,
        instance_buffer_capacity: usize,
    ) {
        let sig = Signature {
            instances_len: visual_world.instances().len(),
            draw_order_len: visual_world.draw_order().len(),
            draw_batches_len: visual_world.draw_batches().len(),
            cached_instance_floats,
            instance_buffer_capacity,
        };

        if !self.should_print(sig) {
            return;
        }

    // ECS summary
    let entities = world.get_all_entities();
    println!("\n=== RenderingInspector @ {stage} ===");
    println!("ECS: entities={}", entities.len());

    self.print_entity_tree(world);

        // VisualWorld summary
        println!(
            "VisualWorld: instances={} draw_order={} draw_batches={} instance_dirty={} draw_cache_dirty={} ",
            visual_world.instances().len(),
            visual_world.draw_order().len(),
            visual_world.draw_batches().len(),
            visual_world.instance_data_dirty(),
            // `dirty_draw_cache` is private; infer: if draw_batches empty but instances exist, likely dirty.
            // Keep this as a best-effort signal.
            (visual_world.instances().len() > 0 && visual_world.draw_batches().is_empty())
        );

        // Draw batches details
        for (i, b) in visual_world.draw_batches().iter().enumerate() {
            println!(
                "  batch[{i}]: material={:?} mesh={:?} start={} count={}",
                material_name(b.material),
                b.mesh,
                b.start,
                b.count
            );
        }

        // Instance buffer summary (CPU-side packed data + GPU capacity)
        let instance_count = visual_world.draw_order().len();
        println!(
            "Instance data: instances={} packed_floats={} (expected={}) gpu_capacity_instances={}",
            instance_count,
            cached_instance_floats,
            instance_count * 16,
            instance_buffer_capacity
        );

        // Print first couple matrices for sanity.
        // We only do a cheap print of translation column (c3) to keep logs manageable.
        if !visual_world.draw_order().is_empty() {
            let preview = visual_world
                .draw_order()
                .iter()
                .take(4)
                .enumerate()
                .map(|(di, &idx)| {
                    let (_r, inst) = visual_world.instances()[idx as usize];
                    let c3 = inst.transform.model[3];
                    (di, idx, c3)
                })
                .collect::<Vec<_>>();

            for (di, idx, c3) in preview {
                println!("  instance_draw[{di}] src_idx={idx}: model.c3 = [{:.3}, {:.3}, {:.3}, {:.3}]", c3[0], c3[1], c3[2], c3[3]);
            }
        }
    }

    /// Print entity -> component -> children component tree.
    ///
    /// This is intentionally verbose and meant for bring-up / debug.
    pub fn print_entity_tree(&self, world: &World) {
        let mut entities = world.get_all_entities();
        entities.sort_by_key(|e| e.id);

        println!("ECS tree:");
        for e in entities {
            println!("  Entity {}", e.id);
            for &root in e.roots() {
                self.print_component_subtree(e, root, 2);
            }
        }
    }

    fn print_component_subtree(
        &self,
        e: &crate::engine::ecs::entity::Entity,
        cid: ComponentId,
        indent: usize,
    ) {
        let Some(c) = e.get_component_by_id(cid) else {
            println!("{:indent$}- <missing component {cid}>", "", indent = indent * 2);
            return;
        };

        let type_name = truncate_from_engine(c.type_name());
        println!(
            "{:indent$}- cid={} type={}",
            "",
            cid,
            type_name,
            indent = indent * 2
        );

        let children = e.children_of(cid);
        for &child in children {
            self.print_component_subtree(e, child, indent + 1);
        }
    }

    /// Print a VisualWorld-only summary (useful when you don't have access to ECS World).
    pub fn print_visuals_only(
        &mut self,
        stage: &str,
        visual_world: &VisualWorld,
        cached_instance_floats: usize,
        instance_buffer_capacity: usize,
    ) {
        let sig = Signature {
            instances_len: visual_world.instances().len(),
            draw_order_len: visual_world.draw_order().len(),
            draw_batches_len: visual_world.draw_batches().len(),
            cached_instance_floats,
            instance_buffer_capacity,
        };

        if !self.should_print(sig) {
            return;
        }

        println!("\n=== RenderingInspector @ {stage} ===");
        println!(
            "VisualWorld: instances={} draw_order={} draw_batches={} instance_dirty={}",
            visual_world.instances().len(),
            visual_world.draw_order().len(),
            visual_world.draw_batches().len(),
            visual_world.instance_data_dirty(),
        );

        for (i, b) in visual_world.draw_batches().iter().enumerate() {
            println!(
                "  batch[{i}]: material={:?} mesh={:?} start={} count={}",
                material_name(b.material),
                b.mesh,
                b.start,
                b.count
            );
        }

        let instance_count = visual_world.draw_order().len();
        println!(
            "Instance data: instances={} packed_floats={} (expected={}) gpu_capacity_instances={}",
            instance_count,
            cached_instance_floats,
            instance_count * 16,
            instance_buffer_capacity
        );

        if !visual_world.draw_order().is_empty() {
            let preview = visual_world
                .draw_order()
                .iter()
                .take(4)
                .enumerate()
                .map(|(di, &idx)| {
                    let (_r, inst) = visual_world.instances()[idx as usize];
                    let c3 = inst.transform.model[3];
                    (di, idx, c3)
                })
                .collect::<Vec<_>>();

            for (di, idx, c3) in preview {
                println!("  instance_draw[{di}] src_idx={idx}: model.c3 = [{:.3}, {:.3}, {:.3}, {:.3}]", c3[0], c3[1], c3[2], c3[3]);
            }
        }
    }
}

fn truncate_from_engine(s: &'static str) -> &'static str {
    // Most of our names look like `little_cat::engine::...`.
    // The user wants everything before `engine::` removed.
    match s.split_once("engine::") {
        Some((_prefix, rest)) => rest,
        None => s,
    }
}

fn material_name(h: MaterialHandle) -> &'static str {
    match h {
        MaterialHandle::UNLIT_FULLSCREEN => "UNLIT_FULLSCREEN",
        MaterialHandle::GRADIENT_BG_XY => "GRADIENT_BG_XY",
        MaterialHandle::UNLIT_MESH => "UNLIT_MESH",
        _ => "UNKNOWN",
    }
}
