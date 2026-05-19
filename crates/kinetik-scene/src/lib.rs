//! Scene and instance graph contracts for Kinetik.

use core::fmt;

use kinetik_core::{InstanceId, KinetikError, KinetikResult};
use kinetik_reflect::{PropertyDescriptor, PropertyType};

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

/// Class-level scene instance metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceClassDescriptor {
    /// Stable class name used by scene instances.
    pub class_name: String,
    /// Human-readable display name.
    pub display_name: String,
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
            properties: Vec::new(),
        })
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
    Ok(
        InstanceClassDescriptor::new(class_name, class_name)?.with_properties(vec![
            PropertyDescriptor::new("Name", "Name", PropertyType::String)
                .expect("built-in Name descriptor should be valid"),
        ]),
    )
}

fn validate_class_name(class_name: &str) -> ClassRegistryResult<()> {
    if class_name.trim().is_empty() {
        return Err(ClassRegistryError::EmptyClassName);
    }
    Ok(())
}

/// Minimal instance record used by the initial scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRecord {
    /// Runtime instance ID.
    pub id: InstanceId,
    /// Human-readable instance name.
    pub name: String,
}

/// Minimal scene placeholder.
#[derive(Debug, Default)]
pub struct Scene {
    instances: Vec<InstanceRecord>,
}

impl Scene {
    /// Creates an empty scene.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    /// Adds a root-level instance and returns its ID.
    pub fn add_root(&mut self, name: impl Into<String>) -> InstanceId {
        let id = InstanceId::new(self.instances.len() as u64 + 1);
        self.instances.push(InstanceRecord {
            id,
            name: name.into(),
        });
        id
    }

    /// Finds an instance by ID.
    ///
    /// # Errors
    ///
    /// Returns [`KinetikError::InvalidHandle`] when the instance ID is not present in this scene.
    pub fn get(&self, id: InstanceId) -> KinetikResult<&InstanceRecord> {
        self.instances
            .iter()
            .find(|instance| instance.id == id)
            .ok_or(KinetikError::InvalidHandle {
                kind: "InstanceId",
                id: id.raw(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_contains_root_and_services_in_order() {
        let registry = InstanceClassRegistry::with_default_scene_classes().unwrap();

        assert_eq!(
            registry.class_names(),
            vec![
                "Game",
                "Workspace",
                "Prefabs",
                "Scripts",
                "UI",
                "Lighting",
                "Audio",
                "Physics",
                "Assets",
                "Packages",
            ]
        );
        assert_eq!(registry.descriptors().len(), 10);
    }

    #[test]
    fn default_registry_supports_lookup_by_class_name() {
        let registry = InstanceClassRegistry::with_default_scene_classes().unwrap();

        let game = registry.get(ROOT_CLASS_NAME).unwrap();
        assert_eq!(game.display_name, "Game");
        assert_eq!(
            game.property("Name").unwrap().value_type,
            PropertyType::String
        );

        let workspace = registry.get("Workspace").unwrap();
        assert_eq!(workspace.class_name, "Workspace");
    }

    #[test]
    fn registry_rejects_duplicate_classes() {
        let mut registry = InstanceClassRegistry::new();
        registry
            .register(InstanceClassDescriptor::new("Part", "Part").unwrap())
            .unwrap();

        assert_eq!(
            registry
                .register(InstanceClassDescriptor::new("Part", "Part").unwrap())
                .unwrap_err(),
            ClassRegistryError::DuplicateClass {
                class_name: "Part".to_owned()
            }
        );
    }

    #[test]
    fn registry_reports_missing_classes() {
        let registry = InstanceClassRegistry::new();

        assert_eq!(
            registry.get("Missing").unwrap_err(),
            ClassRegistryError::UnknownClass {
                class_name: "Missing".to_owned()
            }
        );
    }

    #[test]
    fn class_descriptors_require_class_names() {
        assert_eq!(
            InstanceClassDescriptor::new(" ", "Empty").unwrap_err(),
            ClassRegistryError::EmptyClassName
        );
    }

    #[test]
    fn class_descriptor_property_lookup_uses_canonical_path() {
        let descriptor = InstanceClassDescriptor::new("TransformNode", "Transform Node")
            .unwrap()
            .with_properties(vec![PropertyDescriptor::new(
                "Transform.Position",
                "Position",
                PropertyType::Vec3,
            )
            .unwrap()]);

        assert!(descriptor.property("Transform.Position").is_some());
        assert!(descriptor.property("transform.position").is_none());
    }

    #[test]
    fn add_and_get_root_instance() {
        let mut scene = Scene::new();
        let id = scene.add_root("Workspace");
        assert_eq!(scene.get(id).unwrap().name, "Workspace");
    }
}
