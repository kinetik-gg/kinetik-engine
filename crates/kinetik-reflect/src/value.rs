use core::fmt;

use kinetik_core::{Aabb, Color, InstanceId, Quat, Rect, ResourceId, Transform, Vec2, Vec3, Vec4};

use crate::{PropertyDescriptor, ValueError, ValueResult};

/// Reflected property value type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PropertyType {
    /// UTF-8 text.
    String,
    /// Boolean value.
    Bool,
    /// 32-bit floating point number.
    F32,
    /// 2D vector.
    Vec2,
    /// 3D vector.
    Vec3,
    /// 4D vector.
    Vec4,
    /// Quaternion rotation.
    Quat,
    /// Linear RGBA color.
    Color,
    /// Position, rotation, and scale transform.
    Transform,
    /// 2D rectangle.
    Rect,
    /// 3D axis-aligned bounding box.
    Aabb,
    /// Runtime instance handle.
    InstanceId,
    /// Runtime resource handle.
    ResourceId,
}

impl fmt::Display for PropertyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            Self::String => "String",
            Self::Bool => "Bool",
            Self::F32 => "F32",
            Self::Vec2 => "Vec2",
            Self::Vec3 => "Vec3",
            Self::Vec4 => "Vec4",
            Self::Quat => "Quat",
            Self::Color => "Color",
            Self::Transform => "Transform",
            Self::Rect => "Rect",
            Self::Aabb => "Aabb",
            Self::InstanceId => "InstanceId",
            Self::ResourceId => "ResourceId",
        };
        f.write_str(type_name)
    }
}

/// Reflected property value container.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// UTF-8 text.
    String(String),
    /// Boolean value.
    Bool(bool),
    /// 32-bit floating point number.
    F32(f32),
    /// 2D vector.
    Vec2(Vec2),
    /// 3D vector.
    Vec3(Vec3),
    /// 4D vector.
    Vec4(Vec4),
    /// Quaternion rotation.
    Quat(Quat),
    /// Linear RGBA color.
    Color(Color),
    /// Position, rotation, and scale transform.
    Transform(Transform),
    /// 2D rectangle.
    Rect(Rect),
    /// 3D axis-aligned bounding box.
    Aabb(Aabb),
    /// Runtime instance handle.
    InstanceId(InstanceId),
    /// Runtime resource handle.
    ResourceId(ResourceId),
}

impl PropertyValue {
    /// Creates a neutral default value for a reflected type when one exists.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoTypeDefault`] for handle-like types that do not
    /// have a meaningful neutral value under the non-zero ID policy.
    pub fn type_default(value_type: PropertyType) -> ValueResult<Self> {
        let value = match value_type {
            PropertyType::String => Self::String(String::new()),
            PropertyType::Bool => Self::Bool(false),
            PropertyType::F32 => Self::F32(0.0),
            PropertyType::Vec2 => Self::Vec2(Vec2::default()),
            PropertyType::Vec3 => Self::Vec3(Vec3::default()),
            PropertyType::Vec4 => Self::Vec4(Vec4::default()),
            PropertyType::Quat => Self::Quat(Quat::default()),
            PropertyType::Color => Self::Color(Color::default()),
            PropertyType::Transform => Self::Transform(Transform::default()),
            PropertyType::Rect => Self::Rect(Rect::default()),
            PropertyType::Aabb => Self::Aabb(Aabb::default()),
            PropertyType::InstanceId | PropertyType::ResourceId => {
                return Err(ValueError::NoTypeDefault { value_type });
            }
        };
        Ok(value)
    }

    /// Returns the reflected type of this value.
    #[must_use]
    pub const fn property_type(&self) -> PropertyType {
        match self {
            Self::String(_) => PropertyType::String,
            Self::Bool(_) => PropertyType::Bool,
            Self::F32(_) => PropertyType::F32,
            Self::Vec2(_) => PropertyType::Vec2,
            Self::Vec3(_) => PropertyType::Vec3,
            Self::Vec4(_) => PropertyType::Vec4,
            Self::Quat(_) => PropertyType::Quat,
            Self::Color(_) => PropertyType::Color,
            Self::Transform(_) => PropertyType::Transform,
            Self::Rect(_) => PropertyType::Rect,
            Self::Aabb(_) => PropertyType::Aabb,
            Self::InstanceId(_) => PropertyType::InstanceId,
            Self::ResourceId(_) => PropertyType::ResourceId,
        }
    }

    /// Returns whether this value's type matches the descriptor's type.
    #[must_use]
    pub fn is_compatible_with(&self, descriptor: &PropertyDescriptor) -> bool {
        self.property_type() == descriptor.value_type
    }

    /// Validates this value against a property descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError`] when the descriptor itself is invalid or the value
    /// type does not match the descriptor's reflected type.
    pub fn validate_for_descriptor(&self, descriptor: &PropertyDescriptor) -> ValueResult<()> {
        descriptor
            .validate()
            .map_err(ValueError::InvalidDescriptor)?;
        if self.is_compatible_with(descriptor) {
            return Ok(());
        }
        Err(ValueError::TypeMismatch {
            path: descriptor.path.clone(),
            expected: descriptor.value_type,
            actual: self.property_type(),
        })
    }
}
