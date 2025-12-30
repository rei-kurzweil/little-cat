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

    /// Update a transform component's transform value and notify systems.
    pub fn update_transform(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
        transform: crate::engine::graphics::primitives::Transform,
    ) {
        // Update the transform in the component itself first
        if let Some(transform_comp) = world
            .get_component_by_id_as_mut::<crate::engine::ecs::component::TransformComponent>(component) {
            transform_comp.transform = transform;
        }
        self.transform_changed(world, visuals, component);
    }

    /// Remove/reset a transform component's transform value and notify systems.
    pub fn remove_transform(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        if let Some(transform_comp) = world
            .get_component_by_id_as_mut::<crate::engine::ecs::component::TransformComponent>(component) {
            transform_comp.transform = crate::engine::graphics::primitives::Transform::default();
        }
        self.transform_changed(world, visuals, component);
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

    /// Register a camera component.
    pub fn register_camera(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        let handle = self.camera.register_camera(world, visuals, component);
        // Store the handle in the component
        if let Some(camera_comp) = 
            world.get_component_by_id_as_mut::<crate::engine::ecs::component::CameraComponent>(component) 
        {
            camera_comp.handle = Some(handle);
        }
    }

    /// Make a camera active by its component ID.
    pub fn make_active_camera(
        &mut self,
        _world: &mut World,
        visuals: &mut VisualWorld,
        component: ComponentId,
    ) {
        // Get the camera handle from the component
        if let Some(camera_comp) = 
            _world.get_component_by_id_as::<crate::engine::ecs::component::CameraComponent>(component) 
        {
            if let Some(handle) = camera_comp.handle {
                self.camera.set_active_camera(visuals, handle);
            }
        }
    }
    
    /// Process commands from the command queue.
    pub fn process_commands(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        commands: &mut crate::engine::ecs::CommandQueue,
    ) {
        commands.flush(world, self, visuals);
    }
    
    pub fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState) {
        self.transform.tick(world, visuals, input);
        self.renderable.tick(world, visuals, input);
        self.camera.tick(world, visuals, input);
        self.cursor.tick(world, visuals, input);
    }
}
