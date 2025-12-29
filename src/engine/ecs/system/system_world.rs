use super::World;
use crate::engine::ecs::ComponentId;
use crate::engine::ecs::system::CursorSystem;
use crate::engine::ecs::system::CameraSystem;
use crate::engine::ecs::system::RenderableSystem;
use crate::engine::ecs::system::System;
use crate::engine::ecs::system::TransformSystem;
use crate::engine::graphics::{RenderAssets, Renderer, VisualWorld};
use crate::engine::user_input::InputState;

/// System world that holds and runs all registered systems.
#[derive(Debug, Default)]
pub struct SystemWorld {
    pub cursor: CursorSystem,
    pub camera: CameraSystem,
    pub renderable: RenderableSystem,
    pub transform: TransformSystem,
}

impl SystemWorld {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a CursorComponent instance with the CursorSystem.
    pub fn register_cursor(&mut self,  component: ComponentId) {
        self.cursor.register_cursor(component);
    }

    /// Register a RenderableComponent instance with the RenderableSystem.
    pub fn register_renderable(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        self.renderable
            .register_renderable(world, visuals, component);
    }


    /// Prepare render state before issuing a frame.
    ///
    /// This flushes any pending renderables by uploading meshes and inserting GPU-ready
    /// instances into `VisualWorld`.
    pub fn prepare_render(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        render_assets: &mut RenderAssets,
        renderer: &mut Renderer,
    ) {
        self.renderable
            .flush_pending(world, visuals, render_assets, renderer);
    }

    /// Called when a TransformComponent changes.
    pub fn transform_changed(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        self.transform
            .transform_changed(world, visuals, component);
    }

    /// Called when a TransformComponent changes and we want camera components to react.
    ///
    /// This is intentionally separate from `transform_changed` because camera transforms may not
    /// live under an InstanceComponent (and thus shouldn't go through VisualWorld instance sync).
    pub fn camera_transform_changed(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        let _ = (world, visuals, component);
    }
    
    pub fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState) {
        self.transform.tick(world, visuals, input);
        self.renderable.tick(world, visuals, input);
        self.camera.tick(world, visuals, input);
        self.cursor.tick(world, visuals, input);
    }
}
