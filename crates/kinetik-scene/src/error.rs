use core::fmt;

use kinetik_core::{InstanceGuid, InstanceId};
use kinetik_reflect::PropertyType;

/// Result type for scene hierarchy operations.
pub type SceneResult<T> = Result<T, SceneError>;

/// Errors returned by scene hierarchy operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneError {
    /// Scene already has a root instance.
    DuplicateRoot {
        /// Existing root instance ID.
        root_id: InstanceId,
    },
    /// Runtime instance ID is not present in this scene.
    InvalidInstanceId {
        /// Invalid runtime instance ID.
        id: InstanceId,
    },
    /// Stable instance GUID is not present in this scene.
    InvalidInstanceGuid {
        /// Invalid stable instance GUID.
        guid: InstanceGuid,
    },
    /// Scene did not contain a root instance.
    MissingRoot,
    /// Stable instance GUID appeared more than once.
    DuplicateInstanceGuid {
        /// Duplicate stable instance GUID.
        guid: InstanceGuid,
    },
    /// Class name is not registered in this scene.
    UnknownClass {
        /// Missing class name.
        class_name: String,
    },
    /// Instance name was empty or contained a path separator.
    InvalidInstanceName {
        /// Invalid instance name.
        name: String,
    },
    /// Scene path was invalid or not found.
    InvalidPath {
        /// Invalid or missing scene path.
        path: String,
    },
    /// Root instance cannot be deleted through structural mutation.
    CannotDeleteRoot {
        /// Root instance ID.
        root_id: InstanceId,
    },
    /// Root instance cannot be reparented.
    CannotReparentRoot {
        /// Root instance ID.
        root_id: InstanceId,
    },
    /// Reparenting would create a hierarchy cycle.
    ReparentCycle {
        /// Instance being reparented.
        id: InstanceId,
        /// Invalid target parent.
        new_parent: InstanceId,
    },
    /// Property path is not reflected by the instance class.
    UnknownProperty {
        /// Registered instance class name.
        class_name: String,
        /// Missing canonical property path.
        property_path: String,
    },
    /// Reflected property descriptor was invalid.
    InvalidPropertyDescriptor {
        /// Registered instance class name.
        class_name: String,
        /// Canonical property path.
        property_path: String,
        /// Descriptor validation failure.
        reason: String,
    },
    /// Reflected property type has no neutral default value.
    MissingPropertyDefault {
        /// Canonical property path.
        property_path: String,
        /// Reflected value type.
        value_type: PropertyType,
    },
    /// Reflected property value did not match the descriptor type.
    PropertyTypeMismatch {
        /// Canonical property path.
        property_path: String,
        /// Expected descriptor type.
        expected: PropertyType,
        /// Actual value type.
        actual: PropertyType,
    },
    /// Transform derivation was requested for a non-spatial instance class.
    NonSpatialInstance {
        /// Runtime instance ID.
        id: InstanceId,
        /// Registered instance class name.
        class_name: String,
    },
    /// Bounds were requested for a class that does not produce concrete bounds.
    NoBounds {
        /// Runtime instance ID.
        id: InstanceId,
        /// Registered instance class name.
        class_name: String,
    },
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRoot { root_id } => {
                write!(f, "scene already has a root instance: {root_id}")
            }
            Self::InvalidInstanceId { id } => {
                write!(f, "instance ID is not present in this scene: {id}")
            }
            Self::InvalidInstanceGuid { guid } => {
                write!(f, "instance GUID is not present in this scene: {guid}")
            }
            Self::MissingRoot => f.write_str("scene document must contain a root instance"),
            Self::DuplicateInstanceGuid { guid } => {
                write!(f, "scene document contains duplicate instance GUID: {guid}")
            }
            Self::UnknownClass { class_name } => {
                write!(f, "instance class is not registered: {class_name}")
            }
            Self::InvalidInstanceName { name } => {
                write!(
                    f,
                    "instance name must be non-empty and must not contain '/': {name}"
                )
            }
            Self::InvalidPath { path } => write!(f, "scene path is invalid or not found: {path}"),
            Self::CannotDeleteRoot { root_id } => {
                write!(f, "scene root cannot be deleted: {root_id}")
            }
            Self::CannotReparentRoot { root_id } => {
                write!(f, "scene root cannot be reparented: {root_id}")
            }
            Self::ReparentCycle { id, new_parent } => write!(
                f,
                "instance {id} cannot be reparented under its descendant {new_parent}"
            ),
            Self::UnknownProperty {
                class_name,
                property_path,
            } => write!(
                f,
                "property {property_path} is not reflected by instance class {class_name}"
            ),
            Self::InvalidPropertyDescriptor {
                class_name,
                property_path,
                reason,
            } => write!(
                f,
                "property descriptor {class_name}.{property_path} is invalid: {reason}"
            ),
            Self::MissingPropertyDefault {
                property_path,
                value_type,
            } => write!(
                f,
                "property {property_path} has no default value for reflected type {value_type}"
            ),
            Self::PropertyTypeMismatch {
                property_path,
                expected,
                actual,
            } => write!(
                f,
                "property {property_path} expected value type {expected}, got {actual}"
            ),
            Self::NonSpatialInstance { id, class_name } => write!(
                f,
                "instance {id} of class {class_name} does not have spatial transform properties"
            ),
            Self::NoBounds { id, class_name } => {
                write!(
                    f,
                    "instance {id} of class {class_name} does not have bounds"
                )
            }
        }
    }
}

impl std::error::Error for SceneError {}
