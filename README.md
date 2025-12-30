# Little-Cat-Engine 

small game engine `[obstensively]` for making cats,
using vulkan instanced rendering and several layers to describe game objects:

## Windowing
+ uses winit to make a window and passes the RawDisplayHandle to renderer to render into the window
+ provides user input events and frame loop

## Universe
+ holds all the layers below,
+ and provides simple API to build component trees and add them to the world

## (component) World
+ stores list of components and topology (parent / child relationship between components)
+ components can have subcomponents
+ specific types of components register with SystemWorld and have methods that also call SystemWorld
+ registration / removal and methods of components that affect SystemWorld go through a CommandQueue and get applied after systems.tick() in the update loop.

## SystemWorld
+ handles the behaviors of components
+ can have one system's method invoked and then defer to one or more other systems
+ can call methods on components (via CommandQueue)
    + calls to component methods are applied after all systems have run their tick() method.

#### RenderableSystem
+ keeps a queue of CPUMesh from RenderableComponent that need to be converted to GpuMesh and uploaded into the GPU.

## VisualWorld
+ stores a snapshot of Instances and GpuRenderables
+ and builds cache, sorted by material pipeline and mesh

#### RenderAssets
+ converts `CPUMesh` into `GPUMesh`

## Renderer 
+ displays data from VisualWorld through vulkan

# Components
+ InstanceComponent
  + Required to show a component graphically.
  + InstanceComponent needs a child RenderableComponent to be displayed graphically

+ RenderableComponent
  + Several built-in RenderableComponents are available as special constructors on the impl.
  + TODO: make separate material and geometry components

+ TransformComponent
  + lets you specify a transform for your Renderable Instance 
  + Both Renderable and Transform should be placed under an InstanceComponent as children.

+ CameraComponent
  + add child TransformComponent to move the camera
  + TODO: InputComponent to read InputState and set Transform on TransformComponent on CameraComponent

+ Camera2DComponent
  + add child TransformComponent to move the camera


# Lifecycle

#### Frame loop:
```rust
// in engine::Universe:

/// Game/update step
  pub fn update(&mut self, _dt_sec: f32, _input: &InputState) {
      // each frame,
      // 1. Process input events (handled inside systems for now).
      // 2. Let systems call methods on components,
      //      for example, to update transforms or renderables, which
      //      will update VisualWorld can update draw_batches and give Renderer a snapshot
      self.systems.tick(&mut self.world, &mut self.visuals, _input);
      
      // Process commands after tick so any commands queued during tick are processed in the same frame
      self.systems.process_commands(&mut self.world, &mut self.visuals, &mut self.command_queue);
  }

  pub fn render(&mut self, renderer: &mut graphics::Renderer) {
      // Ensure VisualWorld contains only GPU-ready instances.
      self.systems
          .prepare_render(&mut self.world, &mut self.visuals, &mut self.render_assets, renderer);
      // TODO: rebuild inspector around component graph instead of entities.
      renderer.render_visual_world(&mut self.visuals)
              .expect("render failed");
  }
```
