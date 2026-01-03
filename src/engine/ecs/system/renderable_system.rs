use crate::engine::ecs::ComponentId;
use crate::engine::ecs::component::{RenderableComponent, TransformComponent};

use crate::engine::ecs::system::System;
use crate::engine::ecs::World;
use crate::engine::graphics::{GpuRenderable, VisualWorld};
use crate::engine::graphics::{RenderAssets, MeshUploader};
use crate::engine::user_input::InputState;
use crate::engine::graphics::primitives::{CpuMeshHandle, MaterialHandle};
use std::collections::HashMap;

/// System that registers/updates renderables in the `VisualWorld`.
///
/// Contract / intent:
/// - A `RenderableComponent` is expected to be a *descendant* of a `TransformComponent`.
///   (In practice we attach renderables directly under a transform.)
/// - Each `RenderableComponent` corresponds to exactly one `VisualWorld` instance.
/// - The world-space model matrix for that instance is computed by walking up the component
///   tree and multiplying all ancestor `TransformComponent` model matrices.
#[derive(Debug, Default)]
pub struct RenderableSystem {
    renderables: Vec<ComponentId>,

    /// Renderables that have been discovered/registered in ECS but not yet inserted into
    /// VisualWorld because their GPU mesh isn't ready.
    pending: HashMap<ComponentId, PendingRenderable>,
}

#[derive(Debug, Clone, Copy)]
struct PendingRenderable {
    cpu_mesh: CpuMeshHandle,
    material: MaterialHandle,
    renderable_cid: ComponentId,
}

fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0f32; 4]; 4];
    for c in 0..4 {
        for r in 0..4 {
            out[c][r] = a[0][r] * b[c][0] + a[1][r] * b[c][1] + a[2][r] * b[c][2] + a[3][r] * b[c][3];
        }
    }
    out
}

fn world_model_for_renderable(world: &World, renderable_cid: ComponentId) -> Option<[[f32; 4]; 4]> {
    // Walk up from the renderable and collect all TransformComponents along the way.
    let mut transforms: Vec<[[f32; 4]; 4]> = Vec::new();
    let mut cur = renderable_cid;
    while let Some(parent) = world.parent_of(cur) {
        if let Some(t) = world.get_component_by_id_as::<TransformComponent>(parent) {
            transforms.push(t.transform.model);
        }
        cur = parent;
    }

    if transforms.is_empty() {
        return None;
    }

    // Multiply from root -> leaf.
    transforms.reverse();
    let mut model = transforms[0];
    for m in transforms.into_iter().skip(1) {
        model = mat4_mul(model, m);
    }
    Some(model)
}

impl RenderableSystem {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
            pending: HashMap::new(),
        }
    }

    /// Register a renderable component with this system.
    ///
    /// This is also where we ensure a `VisualWorld` instance exists for it.
    pub fn register_renderable(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        if !self
            .renderables
            .iter()
            .any(|c| *c == component)
        {
            self.renderables.push(component);
        }

        self.register_renderable_from_world(world, visuals, component);
    }

    /// Register a renderable by walking the component graph in `World`.
    pub fn register_renderable_from_world(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        // If it's already registered in VisualWorld, nothing else to do.
        {
            let Some(renderable_comp) = world.get_component_by_id_as::<RenderableComponent>(component) else {
                println!("[RenderableSystem]  -> component is not RenderableComponent somehow");
                return;
            };
            if renderable_comp.get_handle().is_some() {
                return;
            }
        }

        // Defer insertion into VisualWorld until the GPU mesh exists.
        let Some(renderable_comp) = world.get_component_by_id_as::<RenderableComponent>(component) else {
            println!("[RenderableSystem]  -> component is not RenderableComponent somehow");
            return;
        };

        self.pending.insert(
            component,
            PendingRenderable {
                cpu_mesh: renderable_comp.renderable.mesh,
                material: renderable_comp.renderable.material,
                renderable_cid: component,
            },
        );
        println!(
            "[RenderableSystem]  -> pending += 1 (pending_len={}) cpu_mesh={:?} material={:?}",
            self.pending.len(),
            renderable_comp.renderable.mesh,
            renderable_comp.renderable.material
        );

        // Mark draw cache dirty only when we actually insert into visuals.
        let _ = visuals;
    }

    /// Flush any pending renderables by uploading required meshes and inserting only
    /// GPU-ready instances into `VisualWorld`.
    pub fn flush_pending(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        render_assets: &mut RenderAssets,
        uploader: &mut dyn MeshUploader,
    ) {
        // println!(
        //     "[RenderableSystem] flush_pending: pending_len={} visuals.instances={} ",
        //     self.pending.len(),
        //     visuals.instances().len()
        // );
        // Collect keys first to avoid borrow issues.
        let keys: Vec<ComponentId> = self.pending.keys().copied().collect();
        for key in keys {
            let Some(p) = self.pending.get(&key).copied() else {
                continue;
            };

            // Upload/resolve GPU mesh.
            let mesh = match render_assets.gpu_mesh_handle(uploader, p.cpu_mesh) {
                Ok(h) => h,
                Err(err) => {
                    println!("[RenderableSystem]  -> gpu_mesh_handle failed for cpu_mesh={:?}: {:?}", p.cpu_mesh, err);
                    continue;
                }
            };

            let gpu_r = GpuRenderable {
                mesh,
                material: p.material,
            };

            let model = match world_model_for_renderable(world, p.renderable_cid) {
                Some(m) => m,
                None => {
                    self.pending.remove(&key);
                    continue;
                }
            };

            let transform = crate::engine::graphics::primitives::Transform { model, ..Default::default() };
            let handle = visuals.register(p.renderable_cid, gpu_r, transform);

            if let Some(renderable_comp) = world.get_component_by_id_as_mut::<RenderableComponent>(p.renderable_cid) {
                renderable_comp.handle = Some(handle);
            }

            // (If you log ComponentId in a format string, use {:?}.)
            self.pending.remove(&key);
        }
    }
}

impl System for RenderableSystem {
    fn tick(&mut self, _world: &mut World, _visuals: &mut VisualWorld, _input: &InputState, _dt_sec: f32) {
        // Intentionally a no-op for now.
        //
        // Per your architecture: VisualWorld registration happens at component registration time
        // (RenderableComponent::init -> SystemWorld::register_renderable -> RenderableSystem::register_renderable).
        //
        // Later, tick() can be used for per-frame sync (transform updates, material changes, etc.)
        // once we decide how to represent those components and what events/dirty flags we have.
    }
}
