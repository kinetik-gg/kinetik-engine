use kinetik_core::{Aabb, Vec3};

use crate::StaticCollisionWorld;

/// Ray used by the first interaction primitive.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct InteractionRay {
    /// World-space ray origin.
    pub origin: Vec3,
    /// Ray direction. It is normalized when the ray is constructed.
    pub direction: Vec3,
    /// Maximum interaction distance.
    pub max_distance: f32,
}

impl InteractionRay {
    /// Creates an interaction ray.
    ///
    /// # Panics
    ///
    /// Panics when `direction` has no length or `max_distance` is negative.
    #[must_use]
    pub fn new(origin: Vec3, direction: Vec3, max_distance: f32) -> Self {
        assert!(max_distance >= 0.0, "max distance must be non-negative");
        Self {
            origin,
            direction: normalize_vec3(direction),
            max_distance,
        }
    }
}

/// Raycast hit against a static interaction target.
#[derive(Debug, Clone, PartialEq)]
pub struct InteractionHit {
    /// Hit collider label.
    pub label: String,
    /// Distance along the ray.
    pub distance: f32,
    /// World-space hit point.
    pub point: Vec3,
}

/// Returns the nearest static-world hit for an interaction ray.
#[must_use]
pub fn raycast_static_world(
    world: &StaticCollisionWorld,
    ray: InteractionRay,
) -> Option<InteractionHit> {
    world
        .colliders()
        .iter()
        .filter_map(|collider| {
            raycast_aabb(ray, collider.bounds).map(|distance| InteractionHit {
                label: collider.label.clone(),
                distance,
                point: Vec3::new(
                    ray.origin.x + ray.direction.x * distance,
                    ray.origin.y + ray.direction.y * distance,
                    ray.origin.z + ray.direction.z * distance,
                ),
            })
        })
        .min_by(|left, right| left.distance.total_cmp(&right.distance))
}

fn raycast_aabb(ray: InteractionRay, bounds: Aabb) -> Option<f32> {
    let mut t_min: f32 = 0.0;
    let mut t_max = ray.max_distance;
    update_axis_interval(
        ray.origin.x,
        ray.direction.x,
        bounds.min.x,
        bounds.max.x,
        &mut t_min,
        &mut t_max,
    )?;
    update_axis_interval(
        ray.origin.y,
        ray.direction.y,
        bounds.min.y,
        bounds.max.y,
        &mut t_min,
        &mut t_max,
    )?;
    update_axis_interval(
        ray.origin.z,
        ray.direction.z,
        bounds.min.z,
        bounds.max.z,
        &mut t_min,
        &mut t_max,
    )?;
    Some(t_min)
}

fn update_axis_interval(
    origin: f32,
    direction: f32,
    min: f32,
    max: f32,
    t_min: &mut f32,
    t_max: &mut f32,
) -> Option<()> {
    if direction.abs() <= f32::EPSILON {
        return (origin >= min && origin <= max).then_some(());
    }
    let inv_direction = 1.0 / direction;
    let mut axis_min = (min - origin) * inv_direction;
    let mut axis_max = (max - origin) * inv_direction;
    if axis_min > axis_max {
        core::mem::swap(&mut axis_min, &mut axis_max);
    }
    *t_min = (*t_min).max(axis_min);
    *t_max = (*t_max).min(axis_max);
    (t_min <= t_max).then_some(())
}

fn normalize_vec3(value: Vec3) -> Vec3 {
    let length_squared = value
        .x
        .mul_add(value.x, value.y.mul_add(value.y, value.z * value.z));
    assert!(
        length_squared > f32::EPSILON,
        "direction must have non-zero length"
    );
    let inv_length = 1.0 / length_squared.sqrt();
    Vec3::new(
        value.x * inv_length,
        value.y * inv_length,
        value.z * inv_length,
    )
}
