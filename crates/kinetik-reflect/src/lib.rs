//! Class-level reflection metadata for Kinetik.

mod descriptor;
mod error;
mod value;

pub use descriptor::{
    EditorEditability, EditorHint, PlayMutability, PropertyDefault, PropertyDescriptor,
    Scriptability, SerializationPolicy, ValidationRule,
};
pub use error::{DescriptorError, ReflectResult, ValueError, ValueResult};
pub use value::{PropertyType, PropertyValue};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-reflect"
}

#[cfg(test)]
mod tests;
