# cat engine「０.２」

<img width="498" height="400" alt="Screenshot_20260106_094219" src="https://github.com/user-attachments/assets/83c00897-aa61-4520-8756-cd7263289800" />


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
+ stores a snapshot of GpuRenderables
+ and builds cache, sorted by material pipeline, mesh, and texture
  + when ever RenderableSystem or LightSystem (or TransformSystem if involving renderables, lights or cameras) updates. 
  
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
+ Camera2DComponent
+ Camera3DComponent
  + add to TransformComponent to use that transform's model matrix for the camera
  + add to TransformComponent and add that TransformComponent to an InputComponent to control the camera with the keyboard.

```
// input example
InputComponent {
    TransformComponent {
        Camera2DComponent { }
    }
}
```

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



# REPL / CLI

There is a small stdin-driven REPL (processed on the main thread in `Universe::update()`) for inspecting the component tree.

## Commands

- `help` — print commands
- `ls` — list children of the current working component (or roots at `/`)
- `cd <name|index|guid|path>` — change working component
  - `cd /` goes to root
  - `cd ..` goes to parent
  - `cd /7v1:root/8v1:child` walks by `ComponentId` tokens and names
  - `cd <guid>` supports a global jump by GUID
- `pwd` — print a copy-pastable path for the current working component
- `cat [path]` — pretty-print JSON serialization of the subtree
  - `cat` with no args prints from the current working component
  - `cat /` prints the whole scene (all roots)
- `clear` / `cls` — clear the terminal

## Pipes

Pipes use `|` but they pipe *component objects* (ComponentIds), not strings.

- A trailing `|` prints an `ls`-style summary of the piped components.
  - Example: `cat / |`

### `grep`

`grep <pattern>` filters the piped components by matching against component properties (including `name`, `type`, `guid`, and encoded fields), and prints the full serialized value of any matching property.

- Example: `ls | grep color`
- Example: `cat /6v1:input | grep camera`


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


https://github.com/user-attachments/assets/ce4ac311-1087-4792-bec8-5dd012d848f2

