use kinetik_core::{Aabb, InstanceId, Quat, Transform, Vec3};
use kinetik_reflect::{PropertyType, PropertyValue};

use crate::{InstanceRecord, Scene, SceneError, SceneResult};

/// Approved M7 local bounds for `Part`.
pub const PART_LOCAL_BOUNDS: Aabb =
    Aabb::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));

impl Scene {
    /// Returns the local transform for a spatial instance.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the instance is missing, the class is not
    /// spatial, or transform property storage is invalid.
    pub fn local_transform(&self, id: InstanceId) -> SceneResult<Transform> {
        let instance = self.get(id)?;
        self.require_spatial_instance(instance)?;
        local_transform_for_instance(instance)
    }

    /// Returns the deterministic world transform for a spatial instance.
    ///
    /// Non-spatial ancestors contribute identity; spatial ancestors compose in
    /// parent-before-child order.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the instance is missing, the target class is
    /// not spatial, or transform property storage is invalid.
    pub fn world_transform(&self, id: InstanceId) -> SceneResult<Transform> {
        let target = self.get(id)?;
        self.require_spatial_instance(target)?;

        let mut hierarchy = Vec::new();
        let mut current = Some(id);
        while let Some(current_id) = current {
            let instance = self.get(current_id)?;
            hierarchy.push(current_id);
            current = instance.parent;
        }
        hierarchy.reverse();

        let mut transform = Transform::IDENTITY;
        for current_id in hierarchy {
            let instance = self.get(current_id)?;
            if self.is_spatial_instance(instance)? {
                transform = compose_transform(transform, local_transform_for_instance(instance)?);
            }
        }
        Ok(transform)
    }

    /// Returns approved local bounds for a bounded instance.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::NoBounds`] when the instance class has no concrete
    /// authoring bounds in the current M7 contract.
    pub fn local_bounds(&self, id: InstanceId) -> SceneResult<Aabb> {
        let instance = self.get(id)?;
        match instance.class_name.as_str() {
            "Part" => Ok(PART_LOCAL_BOUNDS),
            _ => Err(SceneError::NoBounds {
                id,
                class_name: instance.class_name.clone(),
            }),
        }
    }

    /// Returns deterministic world bounds for a bounded instance.
    ///
    /// World bounds transform all eight local AABB corners through the
    /// instance world transform and take the resulting min/max.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when bounds or world transform derivation fails.
    pub fn world_bounds(&self, id: InstanceId) -> SceneResult<Aabb> {
        Ok(transform_aabb(
            self.local_bounds(id)?,
            self.world_transform(id)?,
        ))
    }
}

pub(crate) fn local_transform_for_instance(instance: &InstanceRecord) -> SceneResult<Transform> {
    Ok(Transform::new(
        vec3_property(instance, "Transform.Position")?,
        quat_property(instance, "Transform.Rotation")?,
        vec3_property(instance, "Transform.Scale")?,
    ))
}
fn vec3_property(instance: &InstanceRecord, property_path: &str) -> SceneResult<Vec3> {
    let Some(value) = instance.properties.get(property_path) else {
        return Err(SceneError::UnknownProperty {
            class_name: instance.class_name.clone(),
            property_path: property_path.to_owned(),
        });
    };
    match value {
        PropertyValue::Vec3(value) => Ok(*value),
        value => Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_owned(),
            expected: PropertyType::Vec3,
            actual: value.property_type(),
        }),
    }
}

fn quat_property(instance: &InstanceRecord, property_path: &str) -> SceneResult<Quat> {
    let Some(value) = instance.properties.get(property_path) else {
        return Err(SceneError::UnknownProperty {
            class_name: instance.class_name.clone(),
            property_path: property_path.to_owned(),
        });
    };
    match value {
        PropertyValue::Quat(value) => Ok(*value),
        value => Err(SceneError::PropertyTypeMismatch {
            property_path: property_path.to_owned(),
            expected: PropertyType::Quat,
            actual: value.property_type(),
        }),
    }
}

pub(crate) fn is_transform_property_path(property_path: &str) -> bool {
    matches!(
        property_path,
        "Transform.Position" | "Transform.Rotation" | "Transform.Scale"
    )
}
fn compose_transform(parent: Transform, local: Transform) -> Transform {
    Transform::new(
        add_vec3(
            parent.position,
            rotate_vec3(parent.rotation, mul_vec3(parent.scale, local.position)),
        ),
        mul_quat(parent.rotation, local.rotation),
        mul_vec3(parent.scale, local.scale),
    )
}

fn transform_aabb(bounds: Aabb, transform: Transform) -> Aabb {
    let corners = aabb_corners(bounds);
    let mut min = transform_point(transform, corners[0]);
    let mut max = min;

    for corner in corners.into_iter().skip(1) {
        let point = transform_point(transform, corner);
        min = min_vec3(min, point);
        max = max_vec3(max, point);
    }

    Aabb::new(min, max)
}

fn aabb_corners(bounds: Aabb) -> [Vec3; 8] {
    [
        Vec3::new(bounds.min.x, bounds.min.y, bounds.min.z),
        Vec3::new(bounds.min.x, bounds.min.y, bounds.max.z),
        Vec3::new(bounds.min.x, bounds.max.y, bounds.min.z),
        Vec3::new(bounds.min.x, bounds.max.y, bounds.max.z),
        Vec3::new(bounds.max.x, bounds.min.y, bounds.min.z),
        Vec3::new(bounds.max.x, bounds.min.y, bounds.max.z),
        Vec3::new(bounds.max.x, bounds.max.y, bounds.min.z),
        Vec3::new(bounds.max.x, bounds.max.y, bounds.max.z),
    ]
}

fn transform_point(transform: Transform, point: Vec3) -> Vec3 {
    add_vec3(
        transform.position,
        rotate_vec3(transform.rotation, mul_vec3(transform.scale, point)),
    )
}

fn add_vec3(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x + right.x, left.y + right.y, left.z + right.z)
}

fn min_vec3(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(
        left.x.min(right.x),
        left.y.min(right.y),
        left.z.min(right.z),
    )
}

fn max_vec3(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(
        left.x.max(right.x),
        left.y.max(right.y),
        left.z.max(right.z),
    )
}

fn mul_vec3(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x * right.x, left.y * right.y, left.z * right.z)
}

fn scale_vec3(value: Vec3, scale: f32) -> Vec3 {
    Vec3::new(value.x * scale, value.y * scale, value.z * scale)
}

fn dot_vec3(left: Vec3, right: Vec3) -> f32 {
    left.x
        .mul_add(right.x, left.y.mul_add(right.y, left.z * right.z))
}

fn cross_vec3(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(
        left.y.mul_add(right.z, -(left.z * right.y)),
        left.z.mul_add(right.x, -(left.x * right.z)),
        left.x.mul_add(right.y, -(left.y * right.x)),
    )
}

fn mul_quat(left: Quat, right: Quat) -> Quat {
    Quat::new(
        left.w.mul_add(
            right.x,
            left.x
                .mul_add(right.w, left.y.mul_add(right.z, -(left.z * right.y))),
        ),
        left.w.mul_add(
            right.y,
            -(left.x * right.z) + left.y * right.w + left.z * right.x,
        ),
        left.w.mul_add(
            right.z,
            left.x
                .mul_add(right.y, -(left.y * right.x) + left.z * right.w),
        ),
        left.w.mul_add(
            right.w,
            -(left.x * right.x) - left.y * right.y - left.z * right.z,
        ),
    )
}

fn rotate_vec3(rotation: Quat, value: Vec3) -> Vec3 {
    let vector = Vec3::new(rotation.x, rotation.y, rotation.z);
    let vector_value_dot = dot_vec3(vector, value);
    let vector_length_squared = dot_vec3(vector, vector);
    add_vec3(
        add_vec3(
            scale_vec3(vector, 2.0 * vector_value_dot),
            scale_vec3(
                value,
                rotation.w.mul_add(rotation.w, -vector_length_squared),
            ),
        ),
        scale_vec3(cross_vec3(vector, value), 2.0 * rotation.w),
    )
}
