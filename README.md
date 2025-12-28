# Little-Cat-Engine 

small game engine `[obstensively]` for making cats,
using vulkan instanced rendering and several layers to describe game objects:

## Windowing
uses winit to make a window and passes the RawDisplayHandle to renderer to render into the window
provides user input events and frame loop

## Universe
holds all the layers below,
and provides simple API to add entities and components

## (component) World
stores entities
entities have components
components can have subcomponents
By default every ecs::Entity starts with one ecs::Component::InstanceComponent.
using `.withComponent()` adds on to that initial component, since typically an entity would have one "root" component.

## SystemWorld
handles the behaviors of components 

## VisualWorld
stores a snapshot of Instances and GpuRenderables
and builds cache, sorted by material pipeline and mesh

## Renderer 
displays data from VisualWorld through vulkan


# Components
+ InstanceComponent
Required to show entity in the world.

+ RenderableComponent
Several built-in RenderableComponents are available as special constructors on the impl.
TODO: make separate material and geometry components

+ TransformComponent
lets you specify a transform for your Renderable Instance 
Both Renderable and Transform should be placed under an InstanceComponent as children.
