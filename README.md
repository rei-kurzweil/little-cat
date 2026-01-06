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
+ and builds cache, sorted by material pipeline, mesh, and texture

#### RenderAssets
+ converts `CPUMesh` into `GPUMesh`

#### TextureSystem
#### LightSystem

## Renderer 
+ displays data from VisualWorld through vulkan

# Components

+ TransformComponent
  + lets position anything in space (and rotate and scale it)
  + affects children:
    + RenderableComponent
    + Camera2DComponent
    + Camera3DComponent
    + PointLightComponent
  + affected by parents:
    + InputComponent (recieves transform input from InputComponent)

+ RenderableComponent
  + Several built-in RenderableComponents are available as special constructors on the impl.
  + TODO: make separate material and geometry components

+ InputComponent
  + Recieves keyboard or other input sources and passes that info to relevant child components
  + TODO: set up key mappings and movement / transform modes beyond the defaults.

+ CameraComponent
  + add child TransformComponent to move the camera
  + TODO: InputComponent to read InputState and set Transform on TransformComponent on CameraComponent

+ Camera2DComponent
  + add child TransformComponent to move the camera

+ ColorComponent
  + Per-instance RGBA tint.
  + Routed into the instanced vertex buffer, so it does not split draw batches.
  + Useful for quick “team color” / debug visualization without creating new materials.

+ UVComponent
  + Supplies UVs for a mesh so shaders can sample textures.

+ TextureComponent
  + References a texture by `uri` (e.g. `"assets/cat-face-neutral.png"`).
  + Loaded/decoded via the `image` crate and uploaded to the GPU.
  + Textures are deduplicated by `uri` (multiple components can share the same GPU texture).
  + Texture affects batching: draw calls are grouped by (material, mesh, texture).

+ PointLightComponent
  + Adds a point light to the scene (fed to the shader via an SSBO).


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
