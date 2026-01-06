pub mod renderable;
pub mod transform;
pub mod camera3d;
pub mod camera2d;
pub mod input;
pub mod point_light;
pub mod lit_voxel;
pub mod uv;
pub mod color;
pub mod texture;

pub use renderable::RenderableComponent;
pub use transform::TransformComponent;
pub use camera3d::Camera3DComponent;
pub use camera2d::Camera2DComponent;
pub use input::InputComponent;
pub use point_light::PointLightComponent;
pub use lit_voxel::LitVoxelComponent;
pub use uv::UVComponent;
pub use color::ColorComponent;
pub use texture::TextureComponent;

/// For now, our "LightComponent" is a point light.
pub type LightComponent = point_light::PointLightComponent;


/// World-owned record for a component payload plus its topology.
///
/// This is the building block of the component-centric ECS: a single flat store of records
/// in `World`, each record carrying its own parent/children handles.

pub struct ComponentNode {
    pub name: &'static str,
    pub component: Box<dyn Component>,
    pub parent: Option<crate::engine::ecs::ComponentId>,
    pub children: Vec<crate::engine::ecs::ComponentId>,
}

impl ComponentNode {
    pub fn new(component: Box<dyn Component>) -> Self {
        let name = component.name();
        Self {
            name,
            component,
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn new_named(name: &'static str, component: Box<dyn Component>) -> Self {
        Self {
            name,
            component,
            parent: None,
            children: Vec::new(),
        }
    }
}

/// Component interface.
/// `init` runs when the component is registered 
pub trait Component: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Short debug/type name for this component kind (e.g. "transform", "camera").
    fn name(&self) -> &'static str;

    fn set_id(
        &mut self,
        _component: crate::engine::ecs::ComponentId,
    ) {
    }

    /// Called when component is added to the World
    fn init(
        &mut self,
        _queue: &mut crate::engine::ecs::CommandQueue,
        _component: crate::engine::ecs::ComponentId,
    ) {
    }

    /// Called when component is removed from the World.
    fn cleanup(
        &mut self,
        _queue: &mut crate::engine::ecs::CommandQueue,
        _component: crate::engine::ecs::ComponentId,
    ) {
    }
}
