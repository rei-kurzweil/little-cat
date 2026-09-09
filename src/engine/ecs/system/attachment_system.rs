use crate::engine::ecs::component::{
    ComponentRef, InputComponent, InputXRGamepadComponent, MountableComponent, QueryRootMode,
    RiderComponent, SerializeComponent, TransformComponent, TransformParentComponent,
    resolve_component_ref,
};
use crate::engine::ecs::system::grabbable_system::ensure_generated_raycastable;
use crate::engine::ecs::system::pointer_system::nearest_ancestor_transform;
use crate::engine::ecs::system::{TransformSystem, ZoneRelation, classify_zone_point};
use crate::engine::ecs::{ComponentId, EventSignal, IntentValue, SignalEmitter, World};
use crate::engine::transform::TransformTrs;
use crate::utils::math::{mat4_inverse, mat4_mul};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
enum SuspendedInput {
    Desktop {
        component: ComponentId,
        enabled: bool,
    },
    XrGamepad {
        component: ComponentId,
        locomotion: bool,
    },
}

#[derive(Debug, Clone, Copy)]
struct ActiveMount {
    rider: ComponentId,
    mountable: ComponentId,
    rider_root: ComponentId,
    rider_anchor: ComponentId,
    mount_anchor: ComponentId,
    dismount_anchor: ComponentId,
    transform_parent: ComponentId,
    original_parent: Option<ComponentId>,
    suspended_input: SuspendedInput,
}

/// Owns live Rider -> Mountable relationships.
///
/// Zones stay passive: they are queried synchronously only when a grip attempts
/// a mount. The per-frame work here is limited to validating active edges.
#[derive(Debug, Default)]
pub struct AttachmentSystem {
    by_rider: HashMap<ComponentId, ActiveMount>,
    rider_by_mountable: HashMap<ComponentId, ComponentId>,
    mounted_this_frame: HashSet<ComponentId>,
}

impl AttachmentSystem {
    pub fn register(
        &mut self,
        world: &mut World,
        mountable: ComponentId,
        emit: &mut dyn SignalEmitter,
    ) {
        if world
            .get_component_by_id_as::<MountableComponent>(mountable)
            .is_some_and(|component| !component.enabled)
        {
            return;
        }
        let Some(owner) = world.parent_of(mountable).filter(|id| {
            world
                .get_component_by_id_as::<TransformComponent>(*id)
                .is_some()
        }) else {
            return;
        };
        ensure_generated_raycastable(world, owner, "mountable_generated_raycastable", emit);
    }

    pub fn begin_frame(&mut self) {
        self.mounted_this_frame.clear();
    }

    /// Temporary first-slice escape gesture: a new grip press from any pointer
    /// in the mounted Rider's scope pops that Rider's current edge.
    pub fn try_dismount_for_pointer(
        &mut self,
        world: &mut World,
        pointer: ComponentId,
        emit: &mut dyn SignalEmitter,
    ) -> bool {
        let Some(rider) = rider_for_pointer(world, pointer) else {
            return false;
        };
        if !self.by_rider.contains_key(&rider) {
            return false;
        }
        if self.mounted_this_frame.contains(&rider) {
            return true;
        }
        self.dismount(world, rider, emit, true)
    }

    /// Try to mount the pointer-associated Rider onto the Mountable resolved
    /// from a ray-hit renderable. Returns true only when the transaction commits.
    pub fn try_mount_from_hit(
        &mut self,
        world: &mut World,
        pointer: ComponentId,
        renderable: ComponentId,
        emit: &mut dyn SignalEmitter,
    ) -> bool {
        let Some(rider_id) = rider_for_pointer(world, pointer) else {
            return false;
        };
        if self.by_rider.contains_key(&rider_id) {
            return false;
        }
        let Some((mountable_id, _owner)) = mountable_for_hit(world, renderable) else {
            return false;
        };
        if self.rider_by_mountable.contains_key(&mountable_id) {
            return false;
        }

        let Some(rider) = world
            .get_component_by_id_as::<RiderComponent>(rider_id)
            .filter(|component| component.enabled)
            .cloned()
        else {
            return false;
        };
        let Some(mountable) = world
            .get_component_by_id_as::<MountableComponent>(mountable_id)
            .filter(|component| component.enabled && component.on_grip)
            .cloned()
        else {
            return false;
        };

        let Some(rider_root) = resolve_required(world, rider_id, rider.movement_root.as_ref())
            .and_then(|id| nearest_ancestor_transform(world, id))
        else {
            return false;
        };
        let Some(rider_anchor) = resolve_required(world, rider_id, rider.anchor.as_ref())
            .and_then(|id| nearest_ancestor_transform(world, id))
        else {
            return false;
        };
        let Some(input) = resolve_required(world, rider_id, rider.input.as_ref()) else {
            return false;
        };
        let Some(entry_zone) = resolve_required(world, mountable_id, mountable.entry_zone.as_ref())
        else {
            return false;
        };
        let Some(mount_anchor) =
            resolve_required(world, mountable_id, mountable.mount_anchor.as_ref())
                .and_then(|id| nearest_ancestor_transform(world, id))
        else {
            return false;
        };
        let Some(dismount_anchor) =
            resolve_required(world, mountable_id, mountable.dismount_anchor.as_ref())
                .and_then(|id| nearest_ancestor_transform(world, id))
        else {
            return false;
        };

        if !is_descendant_or_self(world, rider_root, rider_anchor)
            || is_descendant_or_self(world, rider_root, mount_anchor)
        {
            return false;
        }
        let Some(probe) = TransformSystem::world_position(world, rider_anchor) else {
            return false;
        };
        if !matches!(
            classify_zone_point(world, entry_zone, probe),
            Ok(ZoneRelation::Inside | ZoneRelation::Boundary)
        ) {
            return false;
        }

        let Some(anchor_from_root) = relative_matrix(world, rider_root, rider_anchor) else {
            return false;
        };
        let Some(mount_world) = TransformSystem::world_model(world, mount_anchor) else {
            return false;
        };
        let Some(root_from_anchor) = mat4_inverse(anchor_from_root) else {
            return false;
        };
        let desired_root_world = mat4_mul(mount_world, root_from_anchor);
        let Some(suspended_input) = snapshot_input(world, input) else {
            return false;
        };

        let original_parent = world.parent_of(rider_root);
        let target_guid = match world.get_component_record(mount_anchor) {
            Some(record) => record.guid,
            None => return false,
        };
        let transform_parent = world.add_component_boxed_named(
            "mounted_transform_parent",
            Box::new(
                TransformParentComponent::new().with_target_source(ComponentRef::Guid(target_guid)),
            ),
        );
        let serialize = world.add_component(SerializeComponent::off());
        let _ = world.add_child(transform_parent, serialize);
        if let Some(parent) = original_parent
            && world.add_child(parent, transform_parent).is_err()
        {
            let _ = world.remove_component_subtree(transform_parent);
            return false;
        }
        world.init_component_tree(transform_parent, emit);

        if set_parent_and_world(
            world,
            rider_root,
            Some(transform_parent),
            mount_world,
            desired_root_world,
            emit,
        )
        .is_err()
        {
            let _ = world.set_parent(rider_root, original_parent);
            let _ = world.remove_component_subtree(transform_parent);
            return false;
        }
        suspend_input(world, suspended_input);

        let edge = ActiveMount {
            rider: rider_id,
            mountable: mountable_id,
            rider_root,
            rider_anchor,
            mount_anchor,
            dismount_anchor,
            transform_parent,
            original_parent,
            suspended_input,
        };
        self.by_rider.insert(rider_id, edge);
        self.rider_by_mountable.insert(mountable_id, rider_id);
        self.mounted_this_frame.insert(rider_id);
        true
    }

    pub fn tick(&mut self, world: &mut World, emit: &mut dyn SignalEmitter) {
        let invalid: Vec<ComponentId> = self
            .by_rider
            .iter()
            .filter_map(|(&rider, edge)| (!edge_is_valid(world, *edge)).then_some(rider))
            .collect();
        for rider in invalid {
            let _ = self.dismount(world, rider, emit, false);
        }
    }

    fn dismount(
        &mut self,
        world: &mut World,
        rider: ComponentId,
        emit: &mut dyn SignalEmitter,
        require_dismount_anchor: bool,
    ) -> bool {
        let Some(edge) = self.by_rider.get(&rider).copied() else {
            return false;
        };
        let desired_world =
            TransformSystem::world_model(world, edge.dismount_anchor).and_then(|target| {
                relative_matrix(world, edge.rider_root, edge.rider_anchor)
                    .and_then(mat4_inverse)
                    .map(|root_from_anchor| mat4_mul(target, root_from_anchor))
            });
        if require_dismount_anchor && desired_world.is_none() {
            return false;
        }
        let desired_world =
            desired_world.or_else(|| TransformSystem::world_model(world, edge.rider_root));

        let original_parent = edge
            .original_parent
            .filter(|id| world.get_component_record(*id).is_some());
        if world.get_component_record(edge.rider_root).is_some() {
            let parent_world = effective_parent_world(world, original_parent);
            let restored = desired_world.is_some_and(|desired| {
                set_parent_and_world(
                    world,
                    edge.rider_root,
                    original_parent,
                    parent_world,
                    desired,
                    emit,
                )
                .is_ok()
            });
            if !restored {
                let _ = world.set_parent(edge.rider_root, original_parent);
            }
        }
        restore_input(world, edge.suspended_input);
        if world.get_component_record(edge.transform_parent).is_some() {
            let _ = world.remove_component_subtree(edge.transform_parent);
        }
        self.by_rider.remove(&rider);
        self.rider_by_mountable.remove(&edge.mountable);
        true
    }
}

fn resolve_required(
    world: &World,
    owner: ComponentId,
    source: Option<&ComponentRef>,
) -> Option<ComponentId> {
    resolve_component_ref(
        world,
        source?,
        Some(owner),
        QueryRootMode::ParentScope { levels_up: 1 },
    )
}

fn rider_for_pointer(world: &World, pointer: ComponentId) -> Option<ComponentId> {
    let mut current = Some(pointer);
    while let Some(id) = current {
        if world
            .get_component_by_id_as::<RiderComponent>(id)
            .is_some_and(|rider| rider.enabled)
        {
            return Some(id);
        }
        if world
            .get_component_by_id_as::<TransformComponent>(id)
            .is_some()
        {
            let mut riders = world.children_of(id).iter().copied().filter(|child| {
                world
                    .get_component_by_id_as::<RiderComponent>(*child)
                    .is_some_and(|rider| rider.enabled)
            });
            let rider = riders.next();
            if rider.is_some() && riders.next().is_none() {
                return rider;
            }
            if rider.is_some() {
                return None;
            }
        }
        current = world.parent_of(id);
    }
    None
}

fn mountable_for_hit(world: &World, renderable: ComponentId) -> Option<(ComponentId, ComponentId)> {
    let mut current = Some(renderable);
    while let Some(id) = current {
        if world
            .get_component_by_id_as::<MountableComponent>(id)
            .is_some_and(|mountable| mountable.enabled && mountable.on_grip)
        {
            return nearest_ancestor_transform(world, id).map(|owner| (id, owner));
        }
        if world
            .get_component_by_id_as::<TransformComponent>(id)
            .is_some()
        {
            let found = world.children_of(id).iter().copied().find(|child| {
                world
                    .get_component_by_id_as::<MountableComponent>(*child)
                    .is_some_and(|mountable| mountable.enabled && mountable.on_grip)
            });
            if let Some(mountable) = found {
                return Some((mountable, id));
            }
        }
        current = world.parent_of(id);
    }
    None
}

fn relative_matrix(
    world: &World,
    root: ComponentId,
    descendant: ComponentId,
) -> Option<[[f32; 4]; 4]> {
    let root_world = TransformSystem::world_model(world, root)?;
    let descendant_world = TransformSystem::world_model(world, descendant)?;
    Some(mat4_mul(mat4_inverse(root_world)?, descendant_world))
}

fn is_descendant_or_self(world: &World, ancestor: ComponentId, mut node: ComponentId) -> bool {
    loop {
        if node == ancestor {
            return true;
        }
        let Some(parent) = world.parent_of(node) else {
            return false;
        };
        node = parent;
    }
}

fn effective_parent_world(world: &World, parent: Option<ComponentId>) -> [[f32; 4]; 4] {
    let Some(mut current) = parent else {
        return crate::utils::math::mat4_identity();
    };
    loop {
        if let Some(transform) = world.get_component_by_id_as::<TransformComponent>(current) {
            return transform.transform.matrix_world;
        }
        if let Some(transform_parent) =
            world.get_component_by_id_as::<TransformParentComponent>(current)
            && let Some(target) = transform_parent.resolve_target_component(world)
            && let Some(matrix) = TransformSystem::world_model(world, target)
        {
            return matrix;
        }
        let Some(parent) = world.parent_of(current) else {
            return crate::utils::math::mat4_identity();
        };
        current = parent;
    }
}

fn set_parent_and_world(
    world: &mut World,
    root: ComponentId,
    new_parent: Option<ComponentId>,
    parent_world: [[f32; 4]; 4],
    desired_world: [[f32; 4]; 4],
    emit: &mut dyn SignalEmitter,
) -> Result<(), String> {
    let inverse_parent = mat4_inverse(parent_world).ok_or("attachment parent is singular")?;
    let local = mat4_mul(inverse_parent, desired_world);
    let trs = TransformTrs::from_matrix(local).map_err(|error| error.to_string())?;
    let old_parent = world.parent_of(root);
    world.set_parent(root, new_parent).map_err(str::to_string)?;
    let transform = world
        .get_component_by_id_as_mut::<TransformComponent>(root)
        .ok_or("attachment root is not a transform")?;
    transform.transform.translation = trs.translation;
    transform.transform.rotation = trs.rotation_quat_xyzw;
    transform.transform.scale = trs.scale;
    transform.transform.recompute_model();
    transform.transform.matrix_world = desired_world;
    emit.push_event(
        root,
        EventSignal::ParentChanged {
            child: root,
            old_parent,
            new_parent,
        },
    );
    emit.push_intent_now(
        root,
        IntentValue::UpdateTransform {
            component_id: root,
            translation: trs.translation,
            rotation_quat_xyzw: trs.rotation_quat_xyzw,
            scale: trs.scale,
        },
    );
    Ok(())
}

fn snapshot_input(world: &World, component: ComponentId) -> Option<SuspendedInput> {
    if let Some(input) = world.get_component_by_id_as::<InputComponent>(component) {
        return Some(SuspendedInput::Desktop {
            component,
            enabled: input.enabled,
        });
    }
    world
        .get_component_by_id_as::<InputXRGamepadComponent>(component)
        .map(|input| SuspendedInput::XrGamepad {
            component,
            locomotion: input.locomotion,
        })
}

fn suspend_input(world: &mut World, input: SuspendedInput) {
    match input {
        SuspendedInput::Desktop { component, .. } => {
            if let Some(input) = world.get_component_by_id_as_mut::<InputComponent>(component) {
                input.enabled = false;
            }
        }
        SuspendedInput::XrGamepad { component, .. } => {
            if let Some(input) =
                world.get_component_by_id_as_mut::<InputXRGamepadComponent>(component)
            {
                input.locomotion = false;
            }
        }
    }
}

fn restore_input(world: &mut World, input: SuspendedInput) {
    match input {
        SuspendedInput::Desktop { component, enabled } => {
            if let Some(input) = world.get_component_by_id_as_mut::<InputComponent>(component) {
                input.enabled = enabled;
            }
        }
        SuspendedInput::XrGamepad {
            component,
            locomotion,
        } => {
            if let Some(input) =
                world.get_component_by_id_as_mut::<InputXRGamepadComponent>(component)
            {
                input.locomotion = locomotion;
            }
        }
    }
}

fn edge_is_valid(world: &World, edge: ActiveMount) -> bool {
    world
        .get_component_by_id_as::<RiderComponent>(edge.rider)
        .is_some_and(|rider| rider.enabled)
        && world
            .get_component_by_id_as::<MountableComponent>(edge.mountable)
            .is_some_and(|mountable| mountable.enabled)
        && world.get_component_record(edge.rider_root).is_some()
        && world.get_component_record(edge.rider_anchor).is_some()
        && world.get_component_record(edge.mount_anchor).is_some()
        && world.get_component_record(edge.transform_parent).is_some()
        && world.parent_of(edge.rider_root) == Some(edge.transform_parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::CommandQueue;
    use crate::engine::ecs::component::{PointerComponent, RaycastableComponent, ZoneComponent};

    fn guid_ref(world: &World, component: ComponentId) -> ComponentRef {
        ComponentRef::Guid(world.get_component_record(component).unwrap().guid)
    }

    fn transform(world: &mut World, position: [f32; 3]) -> ComponentId {
        let id = world.add_component(TransformComponent::new().with_position(
            position[0],
            position[1],
            position[2],
        ));
        let component = world
            .get_component_by_id_as_mut::<TransformComponent>(id)
            .unwrap();
        component.transform.matrix_world = component.transform.model;
        id
    }

    struct Fixture {
        world: World,
        rider: ComponentId,
        root: ComponentId,
        anchor: ComponentId,
        input: ComponentId,
        pointer: ComponentId,
        car: ComponentId,
        mountable: ComponentId,
    }

    fn fixture(rider_position: [f32; 3]) -> Fixture {
        let mut world = World::default();
        let root = transform(&mut world, rider_position);
        let anchor = transform(&mut world, rider_position);
        world.add_child(root, anchor).unwrap();
        let pointer = world.add_component(PointerComponent::new());
        world.add_child(anchor, pointer).unwrap();
        let input = world.add_component(InputXRGamepadComponent::new());
        world.add_child(root, input).unwrap();
        let rider = world.add_component(
            RiderComponent::new()
                .anchor(guid_ref(&world, anchor))
                .movement_root(guid_ref(&world, root))
                .input(guid_ref(&world, input)),
        );
        world.add_child(root, rider).unwrap();

        let car = transform(&mut world, [0.0, 0.0, 0.0]);
        let zone = world.add_component(ZoneComponent::sphere(2.0));
        world.add_child(car, zone).unwrap();
        let mount_anchor = transform(&mut world, [10.0, 0.0, 0.0]);
        world.add_child(car, mount_anchor).unwrap();
        let dismount_anchor = transform(&mut world, [3.0, 0.0, 0.0]);
        world.add_child(car, dismount_anchor).unwrap();
        let mountable = world.add_component(
            MountableComponent::new()
                .entry_zone(guid_ref(&world, zone))
                .mount_anchor(guid_ref(&world, mount_anchor))
                .dismount_anchor(guid_ref(&world, dismount_anchor)),
        );
        world.add_child(car, mountable).unwrap();

        Fixture {
            world,
            rider,
            root,
            anchor,
            input,
            pointer,
            car,
            mountable,
        }
    }

    #[test]
    fn mount_then_grip_anywhere_dismounts_and_restores_input() {
        let mut fixture = fixture([0.0, 0.0, 0.0]);
        let mut system = AttachmentSystem::default();
        let mut emit = CommandQueue::new();

        assert!(system.try_mount_from_hit(
            &mut fixture.world,
            fixture.pointer,
            fixture.car,
            &mut emit,
        ));
        assert!(
            !fixture
                .world
                .get_component_by_id_as::<InputXRGamepadComponent>(fixture.input)
                .unwrap()
                .locomotion
        );
        let runtime_parent = fixture.world.parent_of(fixture.root).unwrap();
        assert!(
            fixture
                .world
                .get_component_by_id_as::<TransformParentComponent>(runtime_parent)
                .is_some()
        );
        assert!(!is_descendant_or_self(
            &fixture.world,
            fixture.car,
            fixture.root
        ));

        // Transform propagation would update descendants before the next real
        // input frame. Mirror that one cached result in this focused test.
        let root_world = TransformSystem::world_model(&fixture.world, fixture.root).unwrap();
        let anchor_local = fixture
            .world
            .get_component_by_id_as::<TransformComponent>(fixture.anchor)
            .unwrap()
            .transform
            .model;
        fixture
            .world
            .get_component_by_id_as_mut::<TransformComponent>(fixture.anchor)
            .unwrap()
            .transform
            .matrix_world = mat4_mul(root_world, anchor_local);

        system.begin_frame();
        assert!(system.try_dismount_for_pointer(&mut fixture.world, fixture.pointer, &mut emit,));
        assert_eq!(fixture.world.parent_of(fixture.root), None);
        assert_eq!(
            TransformSystem::world_position(&fixture.world, fixture.root),
            Some([3.0, 0.0, 0.0])
        );
        assert!(
            fixture
                .world
                .get_component_by_id_as::<InputXRGamepadComponent>(fixture.input)
                .unwrap()
                .locomotion
        );
        assert!(!system.by_rider.contains_key(&fixture.rider));
        assert!(!system.rider_by_mountable.contains_key(&fixture.mountable));
    }

    #[test]
    fn mount_rejects_rider_outside_entry_zone() {
        let mut fixture = fixture([3.0, 0.0, 0.0]);
        let mut system = AttachmentSystem::default();
        let mut emit = CommandQueue::new();

        assert!(!system.try_mount_from_hit(
            &mut fixture.world,
            fixture.pointer,
            fixture.car,
            &mut emit,
        ));
        assert!(
            fixture
                .world
                .get_component_by_id_as::<InputXRGamepadComponent>(fixture.input)
                .unwrap()
                .locomotion
        );
        assert_eq!(fixture.world.parent_of(fixture.root), None);
    }

    #[test]
    fn mountable_registration_adds_a_runtime_raycast_marker() {
        let mut fixture = fixture([0.0, 0.0, 0.0]);
        let mut system = AttachmentSystem::default();
        let mut emit = CommandQueue::new();

        system.register(&mut fixture.world, fixture.mountable, &mut emit);

        let marker = fixture
            .world
            .children_of(fixture.car)
            .iter()
            .copied()
            .find(|id| {
                fixture
                    .world
                    .get_component_by_id_as::<RaycastableComponent>(*id)
                    .is_some()
            })
            .expect("generated raycastable");
        assert!(fixture.world.children_of(marker).iter().any(|id| {
            fixture
                .world
                .get_component_by_id_as::<SerializeComponent>(*id)
                .is_some_and(|serialize| !serialize.enabled)
        }));
    }
}
