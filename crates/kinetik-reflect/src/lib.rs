//! Class-level reflection metadata for Kinetik.

use core::fmt;

/// Result type for reflection descriptor operations.
pub type ReflectResult<T> = Result<T, DescriptorError>;

/// Errors returned when descriptor metadata violates the reflection contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorError {
    /// Canonical property path was empty.
    EmptyPath,
    /// Canonical property path was not `PascalCase` dot-separated text.
    InvalidPath {
        /// Invalid path value.
        path: String,
    },
    /// Display name was empty.
    EmptyDisplayName {
        /// Property path being described.
        path: String,
    },
    /// Serialization key was empty.
    EmptySerializationKey {
        /// Property path being described.
        path: String,
    },
    /// A read-only editor property did not explain why it is locked.
    MissingReadOnlyReason {
        /// Property path being described.
        path: String,
    },
}

impl fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => f.write_str("property descriptor path must not be empty"),
            Self::InvalidPath { path } => {
                write!(f, "property descriptor path must be PascalCase: {path}")
            }
            Self::EmptyDisplayName { path } => {
                write!(
                    f,
                    "property descriptor display name must not be empty: {path}"
                )
            }
            Self::EmptySerializationKey { path } => {
                write!(
                    f,
                    "property descriptor serialization key must not be empty: {path}"
                )
            }
            Self::MissingReadOnlyReason { path } => {
                write!(f, "read-only property must include a reason: {path}")
            }
        }
    }
}

impl std::error::Error for DescriptorError {}

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

/// Descriptor-level default value policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PropertyDefault {
    /// Use the reflected type's neutral default.
    #[default]
    TypeDefault,
    /// Preserve an explicit literal for future parsing by the value container.
    Literal(String),
}

/// Inspector and automation presentation hint.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorHint {
    /// Free-form numeric input.
    FreeNumber,
    /// Numeric slider.
    Slider {
        /// Inclusive minimum.
        min: f32,
        /// Inclusive maximum.
        max: f32,
    },
    /// Angle input.
    Angle,
    /// Color picker.
    ColorPicker,
    /// Asset picker.
    AssetPicker,
    /// Instance reference picker.
    InstanceReferencePicker,
    /// Enum or dropdown.
    Enum {
        /// Allowed option labels.
        options: Vec<String>,
    },
    /// Checkbox input.
    Checkbox,
    /// Advanced or collapsed display.
    Advanced,
    /// Runtime-only display.
    RuntimeOnly,
}

/// Validation rule scaffold consumed by future property value validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationRule {
    /// Value is required.
    Required,
    /// String value must not be empty.
    NonEmpty,
    /// Numeric value must be at least `min`.
    MinF32 {
        /// Inclusive minimum.
        min: f32,
    },
    /// Numeric value must be at most `max`.
    MaxF32 {
        /// Inclusive maximum.
        max: f32,
    },
    /// Numeric value must fall inside an inclusive range.
    RangeF32 {
        /// Inclusive minimum.
        min: f32,
        /// Inclusive maximum.
        max: f32,
    },
    /// String value must match one of the allowed values.
    AllowedValues {
        /// Allowed string values.
        values: Vec<String>,
    },
}

/// Whether a reflected property is part of persisted scene or prefab data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SerializationPolicy {
    /// Property is serialized.
    Serialized,
    /// Property is runtime-only and not serialized.
    NotSerialized,
}

/// Whether a reflected property can be edited by editor tooling.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EditorEditability {
    /// Property is editor-editable.
    Editable,
    /// Property is read-only in editor tooling.
    ReadOnly,
}

/// Whether a reflected property is exposed to scripts.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scriptability {
    /// Property is visible to scripts.
    Scriptable,
    /// Property is not visible to scripts.
    NotScriptable,
}

/// Whether a reflected property may be mutated during play mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PlayMutability {
    /// Property may mutate during play mode.
    Mutable,
    /// Property is read-only during play mode.
    ReadOnly,
}

/// Class-level property descriptor shared by runtime, editor, scripting, and automation.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDescriptor {
    /// Canonical `PascalCase` property path.
    pub path: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Reflected value type.
    pub value_type: PropertyType,
    /// Descriptor-level default value policy.
    pub default_value: PropertyDefault,
    /// Stable serialization key.
    pub serialization_key: String,
    /// Serialization behavior.
    pub serialization: SerializationPolicy,
    /// Editor editability behavior.
    pub editor_editability: EditorEditability,
    /// Script exposure behavior.
    pub scriptability: Scriptability,
    /// Play-mode mutability behavior.
    pub play_mutability: PlayMutability,
    /// Reason shown when the property is read-only in editor tooling.
    pub read_only_reason: Option<String>,
    /// Inspector and automation presentation hint.
    pub editor_hint: EditorHint,
    /// Validation rules for future value checking.
    pub validation_rules: Vec<ValidationRule>,
    /// Documentation or help text.
    pub documentation: String,
}

impl PropertyDescriptor {
    /// Creates a descriptor with conservative defaults and validates required fields.
    ///
    /// # Errors
    ///
    /// Returns [`DescriptorError`] when the path is empty, the path is not a
    /// canonical `PascalCase` dot-separated path, or the display name is empty.
    pub fn new(
        path: impl Into<String>,
        display_name: impl Into<String>,
        value_type: PropertyType,
    ) -> ReflectResult<Self> {
        let path = path.into();
        let display_name = display_name.into();
        validate_path(&path)?;
        validate_display_name(&path, &display_name)?;

        Ok(Self {
            serialization_key: path.clone(),
            path,
            display_name,
            value_type,
            default_value: PropertyDefault::TypeDefault,
            serialization: SerializationPolicy::Serialized,
            editor_editability: EditorEditability::Editable,
            scriptability: Scriptability::Scriptable,
            play_mutability: PlayMutability::Mutable,
            read_only_reason: None,
            editor_hint: EditorHint::Advanced,
            validation_rules: Vec::new(),
            documentation: String::new(),
        })
    }

    /// Sets the descriptor-level default value policy.
    #[must_use]
    pub fn with_default_value(mut self, default_value: PropertyDefault) -> Self {
        self.default_value = default_value;
        self
    }

    /// Sets the stable serialization key.
    #[must_use]
    pub fn with_serialization_key(mut self, serialization_key: impl Into<String>) -> Self {
        self.serialization_key = serialization_key.into();
        self
    }

    /// Sets whether this property is serialized.
    #[must_use]
    pub const fn with_serialized(mut self, serialized: bool) -> Self {
        self.serialization = if serialized {
            SerializationPolicy::Serialized
        } else {
            SerializationPolicy::NotSerialized
        };
        self
    }

    /// Sets whether this property is editor-editable.
    #[must_use]
    pub const fn with_editor_editable(mut self, editor_editable: bool) -> Self {
        self.editor_editability = if editor_editable {
            EditorEditability::Editable
        } else {
            EditorEditability::ReadOnly
        };
        self
    }

    /// Sets whether this property is scriptable.
    #[must_use]
    pub const fn with_scriptable(mut self, scriptable: bool) -> Self {
        self.scriptability = if scriptable {
            Scriptability::Scriptable
        } else {
            Scriptability::NotScriptable
        };
        self
    }

    /// Sets whether this property may mutate during play mode.
    #[must_use]
    pub const fn with_mutable_during_play(mut self, mutable_during_play: bool) -> Self {
        self.play_mutability = if mutable_during_play {
            PlayMutability::Mutable
        } else {
            PlayMutability::ReadOnly
        };
        self
    }

    /// Sets the read-only reason.
    #[must_use]
    pub fn with_read_only_reason(mut self, read_only_reason: impl Into<String>) -> Self {
        self.read_only_reason = Some(read_only_reason.into());
        self
    }

    /// Sets the editor hint.
    #[must_use]
    pub fn with_editor_hint(mut self, editor_hint: EditorHint) -> Self {
        self.editor_hint = editor_hint;
        self
    }

    /// Sets validation rules.
    #[must_use]
    pub fn with_validation_rules(mut self, validation_rules: Vec<ValidationRule>) -> Self {
        self.validation_rules = validation_rules;
        self
    }

    /// Sets documentation or help text.
    #[must_use]
    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = documentation.into();
        self
    }

    /// Validates descriptor invariants after builder-style changes.
    ///
    /// # Errors
    ///
    /// Returns [`DescriptorError`] when required strings are empty, paths are
    /// invalid, or a read-only editor property lacks a read-only reason.
    pub fn validate(&self) -> ReflectResult<()> {
        validate_path(&self.path)?;
        validate_display_name(&self.path, &self.display_name)?;
        if self.serialization_key.trim().is_empty() {
            return Err(DescriptorError::EmptySerializationKey {
                path: self.path.clone(),
            });
        }
        if self.editor_editability == EditorEditability::ReadOnly
            && self
                .read_only_reason
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(DescriptorError::MissingReadOnlyReason {
                path: self.path.clone(),
            });
        }
        Ok(())
    }

    /// Returns whether this property is serialized.
    #[must_use]
    pub const fn is_serialized(&self) -> bool {
        matches!(self.serialization, SerializationPolicy::Serialized)
    }

    /// Returns whether this property is editable in editor tooling.
    #[must_use]
    pub const fn is_editor_editable(&self) -> bool {
        matches!(self.editor_editability, EditorEditability::Editable)
    }

    /// Returns whether this property is visible to scripts.
    #[must_use]
    pub const fn is_scriptable(&self) -> bool {
        matches!(self.scriptability, Scriptability::Scriptable)
    }

    /// Returns whether this property may mutate during play mode.
    #[must_use]
    pub const fn is_mutable_during_play(&self) -> bool {
        matches!(self.play_mutability, PlayMutability::Mutable)
    }
}

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-reflect"
}

fn validate_path(path: &str) -> ReflectResult<()> {
    if path.is_empty() {
        return Err(DescriptorError::EmptyPath);
    }
    if path.split('.').all(is_pascal_case_component) {
        return Ok(());
    }
    Err(DescriptorError::InvalidPath {
        path: path.to_owned(),
    })
}

fn validate_display_name(path: &str, display_name: &str) -> ReflectResult<()> {
    if display_name.trim().is_empty() {
        return Err(DescriptorError::EmptyDisplayName {
            path: path.to_owned(),
        });
    }
    Ok(())
}

fn is_pascal_case_component(component: &str) -> bool {
    let mut chars = component.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase() && chars.all(|character| character.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "kinetik-reflect");
    }

    #[test]
    fn descriptor_creation_sets_core_fields() {
        let descriptor =
            PropertyDescriptor::new("Transform.Position", "Position", PropertyType::Vec3)
                .unwrap()
                .with_default_value(PropertyDefault::Literal("0,0,0".to_owned()))
                .with_editor_hint(EditorHint::FreeNumber)
                .with_validation_rules(vec![ValidationRule::Required])
                .with_documentation("Local position.");

        assert_eq!(descriptor.path, "Transform.Position");
        assert_eq!(descriptor.display_name, "Position");
        assert_eq!(descriptor.value_type, PropertyType::Vec3);
        assert_eq!(
            descriptor.default_value,
            PropertyDefault::Literal("0,0,0".to_owned())
        );
        assert_eq!(descriptor.serialization_key, "Transform.Position");
        assert!(descriptor.is_serialized());
        assert!(descriptor.is_editor_editable());
        assert!(descriptor.is_scriptable());
        assert!(descriptor.is_mutable_during_play());
        assert_eq!(descriptor.editor_hint, EditorHint::FreeNumber);
        assert_eq!(descriptor.validation_rules, vec![ValidationRule::Required]);
        assert_eq!(descriptor.documentation, "Local position.");
        descriptor.validate().unwrap();
    }

    #[test]
    fn read_only_descriptors_require_a_reason() {
        let descriptor =
            PropertyDescriptor::new("RuntimeId", "Runtime ID", PropertyType::InstanceId)
                .unwrap()
                .with_editor_editable(false);

        assert_eq!(
            descriptor.validate().unwrap_err(),
            DescriptorError::MissingReadOnlyReason {
                path: "RuntimeId".to_owned()
            }
        );

        descriptor
            .with_read_only_reason("Assigned by the runtime.")
            .validate()
            .unwrap();
    }

    #[test]
    fn invalid_descriptor_cases_are_reported() {
        assert_eq!(
            PropertyDescriptor::new("", "Name", PropertyType::String).unwrap_err(),
            DescriptorError::EmptyPath
        );
        assert_eq!(
            PropertyDescriptor::new("transform.Position", "Position", PropertyType::Vec3)
                .unwrap_err(),
            DescriptorError::InvalidPath {
                path: "transform.Position".to_owned()
            }
        );
        assert_eq!(
            PropertyDescriptor::new("Name", "   ", PropertyType::String).unwrap_err(),
            DescriptorError::EmptyDisplayName {
                path: "Name".to_owned()
            }
        );
        assert_eq!(
            PropertyDescriptor::new("Name", "Name", PropertyType::String)
                .unwrap()
                .with_serialization_key(" ")
                .validate()
                .unwrap_err(),
            DescriptorError::EmptySerializationKey {
                path: "Name".to_owned()
            }
        );
    }
}
