use core::fmt;

use kinetik_core::Vec3;
use kinetik_reflect::{
    EditorHint, PropertyDefault, PropertyDescriptor, PropertyType, PropertyValue,
};

/// Root class name required by the default scene model.
pub const ROOT_CLASS_NAME: &str = "Game";

/// Default service class names scaffolded under [`ROOT_CLASS_NAME`].
pub const DEFAULT_SERVICE_CLASS_NAMES: [&str; 9] = [
    "Workspace",
    "Prefabs",
    "Scripts",
    "UI",
    "Lighting",
    "Audio",
    "Physics",
    "Assets",
    "Packages",
];

/// Approved M7 built-in 3D class names registered after default services.
pub const BUILT_IN_3D_CLASS_NAMES: [&str; 5] = ["Folder", "Node3D", "Part", "Camera3D", "Light3D"];

/// Class-level capabilities used by authoring, validation, and later systems.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InstanceClassCapability {
    /// Can contain child instances as an organization node.
    Container,
    /// Owns editable local transform properties.
    Spatial,
    /// Represents renderable scene content.
    Renderable,
    /// Represents a camera viewpoint.
    Camera,
    /// Represents a light source.
    Light,
}

/// Class-level scene instance metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceClassDescriptor {
    /// Stable class name used by scene instances.
    pub class_name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Stable class capability flags.
    pub capabilities: Vec<InstanceClassCapability>,
    /// Class-level reflected property descriptors.
    pub properties: Vec<PropertyDescriptor>,
}

impl InstanceClassDescriptor {
    /// Creates an instance class descriptor with no class-level properties.
    ///
    /// # Errors
    ///
    /// Returns [`ClassRegistryError::EmptyClassName`] when `class_name` is empty.
    pub fn new(
        class_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> ClassRegistryResult<Self> {
        let class_name = class_name.into();
        validate_class_name(&class_name)?;
        Ok(Self {
            display_name: display_name.into(),
            class_name,
            capabilities: Vec::new(),
            properties: Vec::new(),
        })
    }

    /// Adds class-level capability flags.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: Vec<InstanceClassCapability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Adds class-level reflected property descriptors.
    #[must_use]
    pub fn with_properties(mut self, properties: Vec<PropertyDescriptor>) -> Self {
        self.properties = properties;
        self
    }

    /// Returns a property descriptor by canonical property path.
    #[must_use]
    pub fn property(&self, path: &str) -> Option<&PropertyDescriptor> {
        self.properties
            .iter()
            .find(|descriptor| descriptor.path == path)
    }

    /// Returns whether the descriptor has `capability`.
    #[must_use]
    pub fn has_capability(&self, capability: InstanceClassCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// Result type for scene class registry operations.
pub type ClassRegistryResult<T> = Result<T, ClassRegistryError>;

/// Errors returned by the scene class registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassRegistryError {
    /// Class name was empty.
    EmptyClassName,
    /// Class name was already registered.
    DuplicateClass {
        /// Duplicate class name.
        class_name: String,
    },
    /// Class name was not registered.
    UnknownClass {
        /// Missing class name.
        class_name: String,
    },
}

impl fmt::Display for ClassRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyClassName => f.write_str("instance class name must not be empty"),
            Self::DuplicateClass { class_name } => {
                write!(f, "instance class already registered: {class_name}")
            }
            Self::UnknownClass { class_name } => {
                write!(f, "instance class is not registered: {class_name}")
            }
        }
    }
}

impl std::error::Error for ClassRegistryError {}

/// Deterministic scene instance class registry.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InstanceClassRegistry {
    classes: Vec<InstanceClassDescriptor>,
}

impl InstanceClassRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            classes: Vec::new(),
        }
    }

    /// Creates a registry containing the root and default service classes from ADR 0002.
    ///
    /// # Errors
    ///
    /// Returns [`ClassRegistryError`] if a built-in descriptor violates registry
    /// invariants. This should only happen if the built-in list is changed
    /// incorrectly.
    pub fn with_default_scene_classes() -> ClassRegistryResult<Self> {
        let mut registry = Self::new();
        registry.register(default_class_descriptor(ROOT_CLASS_NAME)?)?;
        for class_name in DEFAULT_SERVICE_CLASS_NAMES {
            registry.register(default_class_descriptor(class_name)?)?;
        }
        for descriptor in built_in_3d_class_descriptors()? {
            registry.register(descriptor)?;
        }
        Ok(registry)
    }

    /// Registers a class descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ClassRegistryError::DuplicateClass`] when the class name is
    /// already registered.
    pub fn register(&mut self, descriptor: InstanceClassDescriptor) -> ClassRegistryResult<()> {
        if self.contains(&descriptor.class_name) {
            return Err(ClassRegistryError::DuplicateClass {
                class_name: descriptor.class_name,
            });
        }
        self.classes.push(descriptor);
        Ok(())
    }

    /// Returns a class descriptor by class name.
    ///
    /// # Errors
    ///
    /// Returns [`ClassRegistryError::UnknownClass`] when the class is not registered.
    pub fn get(&self, class_name: &str) -> ClassRegistryResult<&InstanceClassDescriptor> {
        self.classes
            .iter()
            .find(|descriptor| descriptor.class_name == class_name)
            .ok_or_else(|| ClassRegistryError::UnknownClass {
                class_name: class_name.to_owned(),
            })
    }

    /// Returns whether a class name is registered.
    #[must_use]
    pub fn contains(&self, class_name: &str) -> bool {
        self.classes
            .iter()
            .any(|descriptor| descriptor.class_name == class_name)
    }

    /// Returns registered descriptors in deterministic registration order.
    #[must_use]
    pub fn descriptors(&self) -> &[InstanceClassDescriptor] {
        &self.classes
    }

    /// Returns registered class names in deterministic registration order.
    #[must_use]
    pub fn class_names(&self) -> Vec<&str> {
        self.classes
            .iter()
            .map(|descriptor| descriptor.class_name.as_str())
            .collect()
    }
}

fn default_class_descriptor(class_name: &str) -> ClassRegistryResult<InstanceClassDescriptor> {
    Ok(InstanceClassDescriptor::new(class_name, class_name)?
        .with_properties(vec![name_property_descriptor()]))
}

fn built_in_3d_class_descriptors() -> ClassRegistryResult<Vec<InstanceClassDescriptor>> {
    Ok(vec![
        InstanceClassDescriptor::new("Folder", "Folder")?
            .with_capabilities(vec![InstanceClassCapability::Container])
            .with_properties(vec![
                name_property_descriptor(),
                visible_property_descriptor(),
            ]),
        spatial_descriptor("Node3D", "Node3D", Vec::new())?,
        spatial_descriptor("Part", "Part", vec![InstanceClassCapability::Renderable])?,
        spatial_descriptor(
            "Camera3D",
            "Camera3D",
            vec![InstanceClassCapability::Camera],
        )?,
        spatial_descriptor("Light3D", "Light3D", vec![InstanceClassCapability::Light])?,
    ])
}

fn spatial_descriptor(
    class_name: &str,
    display_name: &str,
    mut extra_capabilities: Vec<InstanceClassCapability>,
) -> ClassRegistryResult<InstanceClassDescriptor> {
    let mut capabilities = vec![InstanceClassCapability::Spatial];
    capabilities.append(&mut extra_capabilities);
    Ok(InstanceClassDescriptor::new(class_name, display_name)?
        .with_capabilities(capabilities)
        .with_properties(shared_spatial_properties()))
}

fn shared_spatial_properties() -> Vec<PropertyDescriptor> {
    vec![
        name_property_descriptor(),
        visible_property_descriptor(),
        PropertyDescriptor::new("Transform.Position", "Position", PropertyType::Vec3)
            .expect("built-in Transform.Position descriptor should be valid")
            .with_editor_hint(EditorHint::Advanced),
        PropertyDescriptor::new("Transform.Rotation", "Rotation", PropertyType::Quat)
            .expect("built-in Transform.Rotation descriptor should be valid")
            .with_editor_hint(EditorHint::Angle),
        PropertyDescriptor::new("Transform.Scale", "Scale", PropertyType::Vec3)
            .expect("built-in Transform.Scale descriptor should be valid")
            .with_default_value(PropertyDefault::Value(PropertyValue::Vec3(Vec3::ONE)))
            .with_editor_hint(EditorHint::Advanced),
    ]
}

fn name_property_descriptor() -> PropertyDescriptor {
    PropertyDescriptor::new("Name", "Name", PropertyType::String)
        .expect("built-in Name descriptor should be valid")
}

fn visible_property_descriptor() -> PropertyDescriptor {
    PropertyDescriptor::new("Visible", "Visible", PropertyType::Bool)
        .expect("built-in Visible descriptor should be valid")
        .with_default_value(PropertyDefault::Value(PropertyValue::Bool(true)))
        .with_editor_hint(EditorHint::Checkbox)
}

fn validate_class_name(class_name: &str) -> ClassRegistryResult<()> {
    if class_name.trim().is_empty() {
        return Err(ClassRegistryError::EmptyClassName);
    }
    Ok(())
}
