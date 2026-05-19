//! Scene and instance graph contracts for Kinetik.

use core::fmt;

use kinetik_core::{InstanceGuid, InstanceId};
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
        }
    }
}

impl std::error::Error for SceneError {}

/// Instance record stored by a scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRecord {
    /// Runtime instance ID.
    pub id: InstanceId,
    /// Stable instance GUID for edit-world identity.
    pub guid: InstanceGuid,
    /// Registered instance class name.
    pub class_name: String,
    /// Human-readable instance name.
    pub name: String,
    /// Parent runtime instance ID.
    pub parent: Option<InstanceId>,
    /// Ordered child runtime instance IDs.
    pub children: Vec<InstanceId>,
}

/// In-memory scene hierarchy.
#[derive(Debug)]
pub struct Scene {
    class_registry: InstanceClassRegistry,
    instances: Vec<InstanceRecord>,
    root: Option<InstanceId>,
    next_id: u64,
    next_guid: u64,
}

impl Scene {
    /// Creates an empty scene.
    ///
    /// # Panics
    ///
    /// Panics only if the built-in default scene class list violates registry
    /// invariants.
    #[must_use]
    pub fn new() -> Self {
        Self::with_class_registry(
            InstanceClassRegistry::with_default_scene_classes()
                .expect("built-in scene classes should be valid"),
        )
    }

    /// Creates an empty scene with a caller-provided class registry.
    #[must_use]
    pub const fn with_class_registry(class_registry: InstanceClassRegistry) -> Self {
        Self {
            class_registry,
            instances: Vec::new(),
            root: None,
            next_id: 1,
            next_guid: 1,
        }
    }

    /// Returns the scene class registry.
    #[must_use]
    pub const fn class_registry(&self) -> &InstanceClassRegistry {
        &self.class_registry
    }

    /// Returns the root instance ID when the scene has a root.
    #[must_use]
    pub const fn root_id(&self) -> Option<InstanceId> {
        self.root
    }

    /// Adds the scene root instance and returns its ID.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the scene already has a root, the class is
    /// not registered, or the instance name is invalid.
    pub fn add_root(
        &mut self,
        class_name: impl Into<String>,
        name: impl Into<String>,
    ) -> SceneResult<InstanceId> {
        if let Some(root_id) = self.root {
            return Err(SceneError::DuplicateRoot { root_id });
        }

        let id = self.create_record(class_name, name, None)?;
        self.root = Some(id);
        Ok(id)
    }

    /// Adds a child instance under `parent` and returns its ID.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the parent is missing, the class is not
    /// registered, or the instance name is invalid.
    pub fn add_child(
        &mut self,
        parent: InstanceId,
        class_name: impl Into<String>,
        name: impl Into<String>,
    ) -> SceneResult<InstanceId> {
        self.index_of(parent)?;
        let id = self.create_record(class_name, name, Some(parent))?;
        let parent_index = self.index_of(parent)?;
        self.instances[parent_index].children.push(id);
        Ok(id)
    }

    /// Finds an instance by runtime ID.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::InvalidInstanceId`] when the instance ID is not present.
    pub fn get(&self, id: InstanceId) -> SceneResult<&InstanceRecord> {
        let index = self.index_of(id)?;
        Ok(&self.instances[index])
    }

    /// Finds an instance by stable GUID.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::InvalidInstanceGuid`] when the GUID is not present.
    pub fn get_by_guid(&self, guid: InstanceGuid) -> SceneResult<&InstanceRecord> {
        self.instances
            .iter()
            .find(|instance| instance.guid == guid)
            .ok_or(SceneError::InvalidInstanceGuid { guid })
    }

    /// Finds an instance by absolute scene path.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::InvalidPath`] when the path is malformed or does
    /// not resolve to an instance.
    pub fn get_by_path(&self, path: &str) -> SceneResult<&InstanceRecord> {
        let id = self.id_by_path(path)?;
        self.get(id)
    }

    /// Returns an absolute scene path for an instance.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::InvalidInstanceId`] when the instance ID is not present.
    pub fn path(&self, id: InstanceId) -> SceneResult<String> {
        let mut names = Vec::new();
        let mut current = Some(id);

        while let Some(current_id) = current {
            let instance = self.get(current_id)?;
            names.push(instance.name.as_str());
            current = instance.parent;
        }

        names.reverse();
        Ok(format!("/{}", names.join("/")))
    }

    /// Returns ordered child IDs for an instance.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::InvalidInstanceId`] when the instance ID is not present.
    pub fn children(&self, id: InstanceId) -> SceneResult<&[InstanceId]> {
        Ok(&self.get(id)?.children)
    }

    fn create_record(
        &mut self,
        class_name: impl Into<String>,
        name: impl Into<String>,
        parent: Option<InstanceId>,
    ) -> SceneResult<InstanceId> {
        let class_name = class_name.into();
        let name = name.into();
        self.validate_class(&class_name)?;
        validate_instance_name(&name)?;

        let id = self.next_instance_id();
        let guid = self.next_instance_guid();
        self.instances.push(InstanceRecord {
            id,
            guid,
            class_name,
            name,
            parent,
            children: Vec::new(),
        });
        Ok(id)
    }

    fn id_by_path(&self, path: &str) -> SceneResult<InstanceId> {
        let parts = path_parts(path)?;
        let root_id = self.root.ok_or_else(|| SceneError::InvalidPath {
            path: path.to_owned(),
        })?;
        let root = self.get(root_id)?;
        if root.name != parts[0] {
            return Err(SceneError::InvalidPath {
                path: path.to_owned(),
            });
        }

        let mut current_id = root_id;
        for part in parts.iter().skip(1) {
            current_id =
                self.child_by_name(current_id, part)
                    .ok_or_else(|| SceneError::InvalidPath {
                        path: path.to_owned(),
                    })?;
        }
        Ok(current_id)
    }

    fn child_by_name(&self, parent: InstanceId, name: &str) -> Option<InstanceId> {
        self.get(parent)
            .ok()?
            .children
            .iter()
            .copied()
            .find(|child_id| self.get(*child_id).is_ok_and(|child| child.name == name))
    }

    fn validate_class(&self, class_name: &str) -> SceneResult<()> {
        if self.class_registry.contains(class_name) {
            return Ok(());
        }
        Err(SceneError::UnknownClass {
            class_name: class_name.to_owned(),
        })
    }

    fn index_of(&self, id: InstanceId) -> SceneResult<usize> {
        self.instances
            .iter()
            .position(|instance| instance.id == id)
            .ok_or(SceneError::InvalidInstanceId { id })
    }

    fn next_instance_id(&mut self) -> InstanceId {
        let id = InstanceId::new(self.next_id);
        self.next_id += 1;
        id
    }

    fn next_instance_guid(&mut self) -> InstanceGuid {
        let guid = InstanceGuid::new(self.next_guid);
        self.next_guid += 1;
        guid
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_instance_name(name: &str) -> SceneResult<()> {
    if name.trim().is_empty() || name.contains('/') {
        return Err(SceneError::InvalidInstanceName {
            name: name.to_owned(),
        });
    }
    Ok(())
}

fn path_parts(path: &str) -> SceneResult<Vec<&str>> {
    if !path.starts_with('/') || path == "/" {
        return Err(SceneError::InvalidPath {
            path: path.to_owned(),
        });
    }

    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(SceneError::InvalidPath {
            path: path.to_owned(),
        });
    }
    Ok(parts)
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
    fn add_and_get_root_instance_by_id_and_guid() {
        let mut scene = Scene::new();
        let id = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let root = scene.get(id).unwrap();

        assert_eq!(scene.root_id(), Some(id));
        assert_eq!(root.guid.raw(), 1);
        assert_eq!(root.class_name, ROOT_CLASS_NAME);
        assert_eq!(root.name, "Game");
        assert_eq!(root.parent, None);
        assert_eq!(scene.get_by_guid(root.guid).unwrap().id, id);
    }

    #[test]
    fn child_ordering_and_paths_are_deterministic() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
        let lighting = scene.add_child(game, "Lighting", "Lighting").unwrap();
        let audio = scene.add_child(game, "Audio", "Audio").unwrap();

        assert_eq!(scene.children(game).unwrap(), &[workspace, lighting, audio]);
        assert_eq!(scene.path(game).unwrap(), "/Game");
        assert_eq!(scene.path(workspace).unwrap(), "/Game/Workspace");
        assert_eq!(scene.path(lighting).unwrap(), "/Game/Lighting");
        assert_eq!(scene.get_by_path("/Game/Audio").unwrap().id, audio);
    }

    #[test]
    fn nested_paths_resolve_through_ordered_children() {
        let mut registry = InstanceClassRegistry::with_default_scene_classes().unwrap();
        registry
            .register(InstanceClassDescriptor::new("Folder", "Folder").unwrap())
            .unwrap();
        let mut scene = Scene::with_class_registry(registry);

        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
        let folder = scene.add_child(workspace, "Folder", "Enemies").unwrap();

        assert_eq!(scene.path(folder).unwrap(), "/Game/Workspace/Enemies");
        assert_eq!(
            scene.get_by_path("/Game/Workspace/Enemies").unwrap().id,
            folder
        );
    }

    #[test]
    fn scene_rejects_duplicate_roots() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();

        assert_eq!(
            scene.add_root(ROOT_CLASS_NAME, "OtherGame").unwrap_err(),
            SceneError::DuplicateRoot { root_id: game }
        );
    }

    #[test]
    fn scene_reports_invalid_handles_and_paths() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();

        assert_eq!(
            scene.get(InstanceId::new(99)).unwrap_err(),
            SceneError::InvalidInstanceId {
                id: InstanceId::new(99)
            }
        );
        assert_eq!(
            scene.get_by_guid(InstanceGuid::new(99)).unwrap_err(),
            SceneError::InvalidInstanceGuid {
                guid: InstanceGuid::new(99)
            }
        );
        assert_eq!(
            scene.get_by_path("Game").unwrap_err(),
            SceneError::InvalidPath {
                path: "Game".to_owned()
            }
        );
        assert_eq!(
            scene.get_by_path("/Game/Missing").unwrap_err(),
            SceneError::InvalidPath {
                path: "/Game/Missing".to_owned()
            }
        );
        assert_eq!(scene.path(game).unwrap(), "/Game");
    }

    #[test]
    fn scene_validates_classes_and_names() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();

        assert_eq!(
            scene.add_child(game, "MissingClass", "Thing").unwrap_err(),
            SceneError::UnknownClass {
                class_name: "MissingClass".to_owned()
            }
        );
        assert_eq!(
            scene.add_child(game, "Workspace", "Bad/Name").unwrap_err(),
            SceneError::InvalidInstanceName {
                name: "Bad/Name".to_owned()
            }
        );
    }
}
