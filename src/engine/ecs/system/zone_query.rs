use crate::engine::ecs::component::{
    CollisionShape, QueryRootMode, TransformComponent, ZoneComponent, resolve_component_ref,
};
use crate::engine::ecs::system::TransformSystem;
use crate::engine::ecs::{ComponentId, World};
use crate::utils::math::{mat4_inverse, mat4_mul_vec4};

const ZONE_EPSILON: f32 = 1.0e-5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneRelation {
    Outside,
    Boundary,
    Inside,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneQueryError {
    NotZone(ComponentId),
    Disabled(ComponentId),
    UnresolvedFrame(ComponentId),
    FrameHasNoTransform {
        zone: ComponentId,
        resolved: ComponentId,
    },
    SingularFrame(ComponentId),
}

pub fn classify_point(
    world: &World,
    zone_id: ComponentId,
    point_world: [f32; 3],
) -> Result<ZoneRelation, ZoneQueryError> {
    let zone = world
        .get_component_by_id_as::<ZoneComponent>(zone_id)
        .ok_or(ZoneQueryError::NotZone(zone_id))?;
    if !zone.enabled {
        return Err(ZoneQueryError::Disabled(zone_id));
    }
    let frame = resolve_zone_frame(world, zone_id, zone)?;
    let world_matrix =
        TransformSystem::world_model(world, frame).ok_or(ZoneQueryError::FrameHasNoTransform {
            zone: zone_id,
            resolved: frame,
        })?;
    let inverse = mat4_inverse(world_matrix).ok_or(ZoneQueryError::SingularFrame(zone_id))?;
    let local = mat4_mul_vec4(
        inverse,
        [point_world[0], point_world[1], point_world[2], 1.0],
    );
    if !local.iter().all(|value| value.is_finite()) || local[3].abs() <= f32::EPSILON {
        return Err(ZoneQueryError::SingularFrame(zone_id));
    }
    let point = [
        local[0] / local[3],
        local[1] / local[3],
        local[2] / local[3],
    ];
    Ok(classify_local_point(zone.shape, point))
}

pub fn zone_has_role(zone: &ZoneComponent, role: &str) -> bool {
    zone.roles.iter().any(|candidate| candidate == role)
}

/// Enumerate enabled zones in stable component-tree order below `root`.
pub fn zones_in_subtree(world: &World, root: ComponentId, role: Option<&str>) -> Vec<ComponentId> {
    fn visit(
        world: &World,
        component: ComponentId,
        role: Option<&str>,
        zones: &mut Vec<ComponentId>,
    ) {
        if let Some(zone) = world.get_component_by_id_as::<ZoneComponent>(component)
            && zone.enabled
            && role.is_none_or(|required| zone_has_role(zone, required))
        {
            zones.push(component);
        }
        for &child in world.children_of(component) {
            visit(world, child, role, zones);
        }
    }

    let mut zones = Vec::new();
    if world.get_component_record(root).is_some() {
        visit(world, root, role, &mut zones);
    }
    zones
}

pub fn resolve_zone_frame(
    world: &World,
    zone_id: ComponentId,
    zone: &ZoneComponent,
) -> Result<ComponentId, ZoneQueryError> {
    let resolved = match &zone.frame_source {
        Some(source) => resolve_component_ref(
            world,
            source,
            Some(zone_id),
            QueryRootMode::ParentScope { levels_up: 1 },
        )
        .ok_or(ZoneQueryError::UnresolvedFrame(zone_id))?,
        None => world
            .parent_of(zone_id)
            .ok_or(ZoneQueryError::UnresolvedFrame(zone_id))?,
    };
    let frame = nearest_transform(world, resolved).ok_or(ZoneQueryError::FrameHasNoTransform {
        zone: zone_id,
        resolved,
    })?;
    let world_matrix =
        TransformSystem::world_model(world, frame).ok_or(ZoneQueryError::FrameHasNoTransform {
            zone: zone_id,
            resolved: frame,
        })?;
    if mat4_inverse(world_matrix).is_none() {
        return Err(ZoneQueryError::SingularFrame(zone_id));
    }
    Ok(frame)
}

fn nearest_transform(world: &World, start: ComponentId) -> Option<ComponentId> {
    let mut current = Some(start);
    while let Some(component) = current {
        if world
            .get_component_by_id_as::<TransformComponent>(component)
            .is_some()
        {
            return Some(component);
        }
        current = world.parent_of(component);
    }
    None
}

fn classify_local_point(shape: CollisionShape, point: [f32; 3]) -> ZoneRelation {
    match shape.normalized() {
        CollisionShape::Cube { half_extents } => {
            let margins = [
                half_extents[0] - point[0].abs(),
                half_extents[1] - point[1].abs(),
                half_extents[2] - point[2].abs(),
            ];
            classify_margins(&margins)
        }
        CollisionShape::Sphere { radius } => {
            let distance = (point[0] * point[0] + point[1] * point[1] + point[2] * point[2]).sqrt();
            classify_margin(radius - distance)
        }
        CollisionShape::CapsuleY {
            radius,
            half_segment,
        } => {
            let closest_y = point[1].clamp(-half_segment, half_segment);
            let dy = point[1] - closest_y;
            let distance = (point[0] * point[0] + dy * dy + point[2] * point[2]).sqrt();
            classify_margin(radius - distance)
        }
    }
}

fn classify_margins(margins: &[f32]) -> ZoneRelation {
    if margins.iter().any(|margin| *margin < -ZONE_EPSILON) {
        ZoneRelation::Outside
    } else if margins.iter().any(|margin| margin.abs() <= ZONE_EPSILON) {
        ZoneRelation::Boundary
    } else {
        ZoneRelation::Inside
    }
}

fn classify_margin(margin: f32) -> ZoneRelation {
    if margin < -ZONE_EPSILON {
        ZoneRelation::Outside
    } else if margin.abs() <= ZONE_EPSILON {
        ZoneRelation::Boundary
    } else {
        ZoneRelation::Inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::component::{ComponentRef, TransformComponent};

    #[test]
    fn transformed_cube_classifies_inside_boundary_and_outside() {
        let mut world = World::default();
        let frame = world.add_component(
            TransformComponent::new()
                .with_position(2.0, 0.0, 0.0)
                .with_rotation_quat([
                    0.0,
                    0.0,
                    std::f32::consts::FRAC_1_SQRT_2,
                    std::f32::consts::FRAC_1_SQRT_2,
                ])
                .with_scale(2.0, 1.0, 1.0),
        );
        let transform = world
            .get_component_by_id_as_mut::<TransformComponent>(frame)
            .unwrap();
        transform.transform.matrix_world = transform.transform.model;
        let zone = world.add_component(ZoneComponent::cube([1.0, 0.5, 0.5]));
        world.add_child(frame, zone).unwrap();

        assert_eq!(
            classify_point(&world, zone, [2.0, 0.0, 0.0]),
            Ok(ZoneRelation::Inside)
        );
        assert_eq!(
            classify_point(&world, zone, [2.0, 2.0, 0.0]),
            Ok(ZoneRelation::Boundary)
        );
        assert_eq!(
            classify_point(&world, zone, [2.0, 2.1, 0.0]),
            Ok(ZoneRelation::Outside)
        );
    }

    #[test]
    fn at_accepts_guid_and_query_refs_in_the_containing_scope() {
        let mut world = World::default();
        let scope = world.add_component(TransformComponent::new());
        let target = world.add_component_boxed_named("target", Box::new(TransformComponent::new()));
        let by_query = world.add_component(
            ZoneComponent::sphere(1.0).at(ComponentRef::Query("[name='target']".into())),
        );
        let guid = world.get_component_record(target).unwrap().guid;
        let by_guid = world.add_component(ZoneComponent::sphere(1.0).at(ComponentRef::Guid(guid)));
        world.add_child(scope, target).unwrap();
        world.add_child(scope, by_query).unwrap();
        world.add_child(scope, by_guid).unwrap();

        assert_eq!(
            classify_point(&world, by_query, [0.0, 0.0, 0.0]),
            Ok(ZoneRelation::Inside)
        );
        assert_eq!(
            classify_point(&world, by_guid, [0.0, 0.0, 0.0]),
            Ok(ZoneRelation::Inside)
        );
    }

    #[test]
    fn disabled_and_unresolved_zones_fail_explicitly() {
        let mut world = World::default();
        let frame = world.add_component(TransformComponent::new());
        let disabled = world.add_component(ZoneComponent::sphere(1.0).enabled(false));
        let unresolved = world.add_component(
            ZoneComponent::sphere(1.0).at(ComponentRef::Query("[name='missing']".into())),
        );
        world.add_child(frame, disabled).unwrap();
        world.add_child(frame, unresolved).unwrap();

        assert_eq!(
            classify_point(&world, disabled, [0.0; 3]),
            Err(ZoneQueryError::Disabled(disabled))
        );
        assert_eq!(
            classify_point(&world, unresolved, [0.0; 3]),
            Err(ZoneQueryError::UnresolvedFrame(unresolved))
        );
    }

    #[test]
    fn sphere_and_capsule_classify_curved_boundaries() {
        let mut world = World::default();
        let frame = world.add_component(TransformComponent::new());
        let sphere = world.add_component(ZoneComponent::sphere(1.0));
        let capsule = world.add_component(ZoneComponent::capsule_y(0.5, 1.0));
        world.add_child(frame, sphere).unwrap();
        world.add_child(frame, capsule).unwrap();

        assert_eq!(
            classify_point(&world, sphere, [1.0, 0.0, 0.0]),
            Ok(ZoneRelation::Boundary)
        );
        assert_eq!(
            classify_point(&world, sphere, [1.01, 0.0, 0.0]),
            Ok(ZoneRelation::Outside)
        );
        assert_eq!(
            classify_point(&world, capsule, [0.0, 1.5, 0.0]),
            Ok(ZoneRelation::Boundary)
        );
        assert_eq!(
            classify_point(&world, capsule, [0.0, 0.0, 0.0]),
            Ok(ZoneRelation::Inside)
        );
    }

    #[test]
    fn singular_zone_frame_is_rejected() {
        let mut world = World::default();
        let frame = world.add_component(TransformComponent::new().with_scale(0.0, 1.0, 1.0));
        let transform = world
            .get_component_by_id_as_mut::<TransformComponent>(frame)
            .unwrap();
        transform.transform.matrix_world = transform.transform.model;
        let zone = world.add_component(ZoneComponent::cube([1.0; 3]));
        world.add_child(frame, zone).unwrap();

        assert_eq!(
            classify_point(&world, zone, [0.0; 3]),
            Err(ZoneQueryError::SingularFrame(zone))
        );
    }

    #[test]
    fn subtree_enumeration_is_stable_role_filtered_and_skips_disabled_zones() {
        let mut world = World::default();
        let root = world.add_component(TransformComponent::new());
        let legs = world.add_component(ZoneComponent::sphere(1.0).role("mount"));
        let mouth = world.add_component(ZoneComponent::sphere(1.0).role("socket"));
        let disabled = world.add_component(ZoneComponent::sphere(1.0).role("mount").enabled(false));
        world.add_child(root, legs).unwrap();
        world.add_child(root, mouth).unwrap();
        world.add_child(root, disabled).unwrap();

        assert_eq!(zones_in_subtree(&world, root, None), vec![legs, mouth]);
        assert_eq!(zones_in_subtree(&world, root, Some("mount")), vec![legs]);
    }
}
