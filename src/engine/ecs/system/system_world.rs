use super::World;
use crate::engine::ecs::entity::{ComponentId, EntityId};
use crate::engine::ecs::system::CursorSystem;
use crate::engine::ecs::system::RenderableSystem;
use crate::engine::ecs::system::System;
use crate::engine::ecs::system::TransformSystem;
use crate::engine::graphics::{RenderAssets, Renderer, VisualWorld};
use crate::engine::user_input::InputState;
use crate::engine::ecs::entity::Entity;

/// System world that holds and runs all registered systems.
#[derive(Debug, Default)]
pub struct SystemWorld {
    pub cursor: CursorSystem,
    pub renderable: RenderableSystem,
    pub transform: TransformSystem,
}

impl SystemWorld {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a CursorComponent instance with the CursorSystem.
    pub fn register_cursor(&mut self, entity: EntityId, component: ComponentId) {
        self.cursor.register_cursor(entity, component);
    }

    /// Register a RenderableComponent instance with the RenderableSystem.
    pub fn register_renderable(
        &mut self,
        world: &mut World,
        visuals: &mut VisualWorld,
        entity: EntityId,
        component: ComponentId,
    ) {
        self.renderable
            .register_renderable(world, visuals, entity, component);
    }

    /// Register a RenderableComponent when you already have access to the `Entity`.
    ///
    /// This avoids needing the entity to already be inserted into `World`.
    pub fn register_renderable_from_entity(
        &mut self,
        visuals: &mut VisualWorld,
        ent: &mut Entity,
        component: ComponentId,
    ) {
        self.renderable
            .register_renderable_from_entity(visuals, ent, component);
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
        entity: EntityId,
        component: ComponentId,
    ) {
        self.transform
            .transform_changed(world, visuals, entity, component);
    }
    
    pub fn tick(&mut self, world: &mut World, visuals: &mut VisualWorld, input: &InputState) {
        self.transform.tick(world, visuals, input);
        self.renderable.tick(world, visuals, input);
        self.cursor.tick(world, visuals, input);
    }
}
