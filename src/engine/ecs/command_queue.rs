/**
 * Queue for commands (methods on components) 
 * which reach systems after all components have been interacted, before rendering the next frame.
 * 
 */

pub struct CommandQueue {
    commands: Vec<ComponentCommand>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    
    /// Queue a register renderable command.
    pub fn queue_register_renderable(
        &mut self,
        component_id: crate::engine::ecs::ComponentId,
    ) {
        self.commands.push(ComponentCommand {
            component_id,
            command: Command::REGISTER_RENDERABLE {
                component_id,
            },
        });
    }

    /// Queue a register transform command.
    pub fn queue_register_transform(
        &mut self,
        component_id: crate::engine::ecs::ComponentId,
    ) {
        self.commands.push(ComponentCommand {
            component_id,
            command: Command::REGISTER_TRANSFORM {
                component_id,
            },
        });
    }

    /// Queue an update transform command.
    pub fn queue_update_transform(
        &mut self,
        component_id: crate::engine::ecs::ComponentId,
        transform: crate::engine::graphics::primitives::Transform,
    ) {
        self.commands.push(ComponentCommand {
            component_id,
            command: Command::UPDATE_TRANSFORM {
                component_id,
                transform,
            },
        });
    }


    /// Queue a register camera command.
    pub fn queue_register_camera(
        &mut self,
        component_id: crate::engine::ecs::ComponentId,
    ) {
        self.commands.push(ComponentCommand {
            component_id,
            command: Command::REGISTER_CAMERA { component_id },
        });
    }

    /// Queue a register camera2d command.
    pub fn queue_register_camera2d(
        &mut self,
        component_id: crate::engine::ecs::ComponentId,
    ) {
        self.commands.push(ComponentCommand {
            component_id,
            command: Command::REGISTER_CAMERA2D { component_id },
        });
    }

    /// Queue a make active camera command.
    pub fn queue_make_active_camera(
        &mut self,
        component_id: crate::engine::ecs::ComponentId,
    ) {
        self.commands.push(ComponentCommand {
            component_id,
            command: Command::MAKE_ACTIVE_CAMERA { component_id },
        });
    }

    /// Queue a register input command.
    pub fn queue_register_input(
        &mut self,
        component_id: crate::engine::ecs::ComponentId,
    ) {
        self.commands.push(ComponentCommand {
            component_id,
            command: Command::REGISTER_INPUT { component_id },
        });
    }

    /// Flush all queued commands, executing them through the systems.
    pub fn flush(
        &mut self,
        world: &mut crate::engine::ecs::World,
        systems: &mut crate::engine::ecs::system::SystemWorld,
        visuals: &mut crate::engine::graphics::VisualWorld,
    ) {
        let commands = std::mem::take(&mut self.commands);
        for cmd in commands {
            match cmd.command {
                Command::REGISTER_TRANSFORM { component_id } => {
                    systems.transform_changed(world, visuals, component_id);
                }
                Command::UPDATE_TRANSFORM { component_id, transform } => {
                    systems.update_transform(world, visuals, component_id, transform);
                }
                Command::REMOVE_TRANSFORM { component_id } => {
                    systems.remove_transform(world, visuals, component_id);
                }
                
                Command::REGISTER_INSTANCE { component_id: _ } => {
                    // TODO: implement when needed
                }
                Command::REMOVE_INSTANCE { component_id: _ } => {
                    // TODO: implement when needed
                }
                Command::REGISTER_CAMERA { component_id } => {
                    systems.register_camera(world, visuals, component_id);
                }
                Command::REGISTER_CAMERA2D { component_id } => {
                    systems.register_camera2d(world, visuals, component_id);
                }
                Command::MAKE_ACTIVE_CAMERA { component_id } => {
                    systems.make_active_camera(world, visuals, component_id);
                }
                Command::REGISTER_CURSOR { component_id: _ } => {
                    // TODO: implement when needed
                }
                Command::REGISTER_INPUT { component_id } => {
                    systems.register_input(component_id);
                }
                Command::REMOVE_CURSOR { component_id: _ } => {
                    // TODO: implement when needed
                }
                Command::REGISTER_RENDERABLE { component_id } => {
                    systems.register_renderable(world, visuals, component_id);
                }
                Command::REMOVE_RENDERABLE { component_id: _ } => {
                    // TODO: implement when needed
                }
                Command::REMOVE_CAMERA { component_id: _ } => {
                    // TODO: implement when needed
                }
            }
        }
    }
}

pub struct ComponentCommand {
    component_id: crate::engine::ecs::ComponentId,
    command: Command,
    //
}

enum Command {
    REGISTER_INSTANCE {
        component_id: crate::engine::ecs::ComponentId,
    },
    REGISTER_RENDERABLE {
        component_id: crate::engine::ecs::ComponentId,
    },
    REGISTER_TRANSFORM {
        component_id: crate::engine::ecs::ComponentId,
    },
    REGISTER_CURSOR {
        component_id: crate::engine::ecs::ComponentId,
    },
    REGISTER_INPUT {
        component_id: crate::engine::ecs::ComponentId,
    },
    REGISTER_CAMERA {
        component_id: crate::engine::ecs::ComponentId,
    },
    REGISTER_CAMERA2D {
        component_id: crate::engine::ecs::ComponentId,
    },

    REMOVE_INSTANCE {
        component_id: crate::engine::ecs::ComponentId,
    },
    REMOVE_RENDERABLE {
        component_id: crate::engine::ecs::ComponentId,
    },
    REMOVE_TRANSFORM {
        component_id: crate::engine::ecs::ComponentId,
    },
    REMOVE_CURSOR {
        component_id: crate::engine::ecs::ComponentId,
    },
    REMOVE_CAMERA {
        component_id: crate::engine::ecs::ComponentId,
    },

    UPDATE_TRANSFORM {
        component_id: crate::engine::ecs::ComponentId,
        transform: crate::engine::graphics::primitives::Transform,
    },

    MAKE_ACTIVE_CAMERA {
        component_id: crate::engine::ecs::ComponentId,
    },
}