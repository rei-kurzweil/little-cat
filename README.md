# Little-Cat-Engine 

small game engine `[obstensively]` for making cats,
using vulkan instanced rendering and several layers to describe game objects:

## Windowing
+ uses winit to make a window and passes the RawDisplayHandle to renderer to render into the window
+ provides user input events and frame loop

## Universe
+ holds all the layers below,
+ and provides simple API to add entities and components

## (component) World
+ stores list of components and topology (parent / child relationship between components)
+ components can have subcomponents
+ specific types of components register with SystemWorld and have methods that also call SystemWorld
+ registration / removal and methods of components that affect SystemWorld go through a CommandQueue and get applied after systems.tick() in the update loop.

## SystemWorld
+ handles the behaviors of components 

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
  + Required to show entity in the world.

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


