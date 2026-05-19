//! Scene and instance graph contracts for Kinetik.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use kinetik_core::{InstanceGuid, InstanceId, Vec3};
use kinetik_reflect::{
    EditorHint, PropertyDefault, PropertyDescriptor, PropertyType, PropertyValue, ValueError,
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
        }
    }
}

impl std::error::Error for SceneError {}

/// Instance record stored by a scene.
#[derive(Debug, Clone, PartialEq)]
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
    /// Reflected property values keyed by canonical property path.
    pub properties: BTreeMap<String, PropertyValue>,
}

/// Dependency-free `.ktscene` document contract.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneDocument {
    /// Root serialized instance tree.
    pub root: SceneInstanceDocument,
}

impl SceneDocument {
    /// Creates a scene document from a root instance tree.
    #[must_use]
    pub const fn new(root: SceneInstanceDocument) -> Self {
        Self { root }
    }
}

/// Serialized instance tree contract used by scene and prefab documents.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneInstanceDocument {
    /// Stable serialized instance identity.
    pub guid: InstanceGuid,
    /// Registered class name.
    pub class_name: String,
    /// Human-readable instance name.
    pub name: String,
    /// Reflected property values keyed by canonical property path.
    pub properties: BTreeMap<String, PropertyValue>,
    /// Ordered child instance documents.
    pub children: Vec<SceneInstanceDocument>,
}

impl SceneInstanceDocument {
    /// Creates a serialized instance document.
    #[must_use]
    pub fn new(guid: InstanceGuid, class_name: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            guid,
            class_name: class_name.into(),
            name: name.into(),
            properties: BTreeMap::new(),
            children: Vec::new(),
        }
    }

    /// Sets reflected property values.
    #[must_use]
    pub fn with_properties(mut self, properties: BTreeMap<String, PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    /// Sets ordered children.
    #[must_use]
    pub fn with_children(mut self, children: Vec<Self>) -> Self {
        self.children = children;
        self
    }
}

/// Scene-local structural mutation batch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneMutationQueue {
    mutations: Vec<SceneMutation>,
}

impl SceneMutationQueue {
    /// Creates an empty mutation queue.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    /// Queues a structural mutation.
    pub fn push(&mut self, mutation: SceneMutation) {
        self.mutations.push(mutation);
    }

    /// Queues root instance creation.
    pub fn create_root(&mut self, class_name: impl Into<String>, name: impl Into<String>) {
        self.push(SceneMutation::Create {
            parent: None,
            class_name: class_name.into(),
            name: name.into(),
        });
    }

    /// Queues child instance creation.
    pub fn create_child(
        &mut self,
        parent: InstanceId,
        class_name: impl Into<String>,
        name: impl Into<String>,
    ) {
        self.push(SceneMutation::Create {
            parent: Some(parent),
            class_name: class_name.into(),
            name: name.into(),
        });
    }

    /// Queues instance deletion.
    pub fn delete(&mut self, id: InstanceId) {
        self.push(SceneMutation::Delete { id });
    }

    /// Queues instance rename.
    pub fn rename(&mut self, id: InstanceId, name: impl Into<String>) {
        self.push(SceneMutation::Rename {
            id,
            name: name.into(),
        });
    }

    /// Queues instance reparenting.
    pub fn reparent(&mut self, id: InstanceId, new_parent: InstanceId) {
        self.push(SceneMutation::Reparent { id, new_parent });
    }

    /// Returns queued mutations in deterministic apply order.
    #[must_use]
    pub fn mutations(&self) -> &[SceneMutation] {
        &self.mutations
    }

    /// Returns the number of queued mutations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.mutations.len()
    }

    /// Returns whether the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }
}

/// Scene-local structural mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneMutation {
    /// Create a root or child instance.
    Create {
        /// Optional parent. `None` creates the scene root.
        parent: Option<InstanceId>,
        /// Registered class name.
        class_name: String,
        /// Instance name.
        name: String,
    },
    /// Delete an instance and its descendants.
    Delete {
        /// Instance to delete.
        id: InstanceId,
    },
    /// Rename an instance.
    Rename {
        /// Instance to rename.
        id: InstanceId,
        /// New instance name.
        name: String,
    },
    /// Move an instance under a new parent.
    Reparent {
        /// Instance to move.
        id: InstanceId,
        /// Target parent instance.
        new_parent: InstanceId,
    },
}

/// Result produced by applying a scene structural mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneMutationResult {
    /// Instance was created.
    Created {
        /// Created instance ID.
        id: InstanceId,
    },
    /// Instance and descendants were deleted.
    Deleted {
        /// Deleted root instance ID.
        id: InstanceId,
        /// Deleted instance IDs in deterministic parent-before-child order.
        deleted_ids: Vec<InstanceId>,
    },
    /// Instance was renamed.
    Renamed {
        /// Renamed instance ID.
        id: InstanceId,
    },
    /// Instance was reparented.
    Reparented {
        /// Reparented instance ID.
        id: InstanceId,
        /// Previous parent.
        old_parent: Option<InstanceId>,
        /// New parent.
        new_parent: InstanceId,
    },
}

/// In-memory scene hierarchy.
#[derive(Debug, Clone)]
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

    /// Creates the default ADR 0002 scene hierarchy.
    ///
    /// The default hierarchy is rooted at `Game` and contains the visible
    /// top-level service instances in [`DEFAULT_SERVICE_CLASS_NAMES`] order.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] if built-in scene classes or service names violate
    /// scene hierarchy invariants.
    pub fn default_scene() -> SceneResult<Self> {
        let mut scene = Self::new();
        let root = scene.add_root(ROOT_CLASS_NAME, ROOT_CLASS_NAME)?;
        for class_name in DEFAULT_SERVICE_CLASS_NAMES {
            scene.add_child(root, class_name, class_name)?;
        }
        Ok(scene)
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

    /// Returns reflected properties for an instance.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::InvalidInstanceId`] when the instance ID is not present.
    pub fn properties(&self, id: InstanceId) -> SceneResult<&BTreeMap<String, PropertyValue>> {
        Ok(&self.get(id)?.properties)
    }

    /// Returns a reflected property value for an instance.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the instance is missing or the property is
    /// not reflected by the instance class.
    pub fn get_property(&self, id: InstanceId, property_path: &str) -> SceneResult<&PropertyValue> {
        let instance = self.get(id)?;
        self.property_descriptor_for_class(&instance.class_name, property_path)?;
        instance
            .properties
            .get(property_path)
            .ok_or_else(|| SceneError::UnknownProperty {
                class_name: instance.class_name.clone(),
                property_path: property_path.to_owned(),
            })
    }

    /// Sets a reflected property value for an instance.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the instance is missing, the property is
    /// unknown, or the value type does not match the reflected descriptor.
    pub fn set_property(
        &mut self,
        id: InstanceId,
        property_path: &str,
        value: PropertyValue,
    ) -> SceneResult<()> {
        let index = self.index_of(id)?;
        let class_name = self.instances[index].class_name.clone();
        {
            let descriptor = self.property_descriptor_for_class(&class_name, property_path)?;
            value
                .validate_for_descriptor(descriptor)
                .map_err(|error| property_value_error(&class_name, property_path, error))?;
        }

        if let ("Name", PropertyValue::String(name)) = (property_path, &value) {
            validate_instance_name(name)?;
        }

        self.instances[index]
            .properties
            .insert(property_path.to_owned(), value);

        if property_path == "Name" {
            let Some(PropertyValue::String(name)) = self.instances[index].properties.get("Name")
            else {
                unreachable!("Name was validated as a string property");
            };
            let name = name.clone();
            self.instances[index].name = name;
        }

        Ok(())
    }

    /// Converts this scene into a deterministic dependency-free scene document.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::MissingRoot`] when the scene has no root.
    pub fn to_document(&self) -> SceneResult<SceneDocument> {
        let root_id = self.root.ok_or(SceneError::MissingRoot)?;
        Ok(SceneDocument::new(self.instance_to_document(root_id)?))
    }

    /// Creates a scene from a dependency-free scene document and class registry.
    ///
    /// Runtime instance IDs are assigned deterministically in parent-before-child
    /// document order. Serialized GUIDs are preserved.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the document contains duplicate GUIDs,
    /// unknown classes, invalid names, or invalid property values.
    pub fn from_document(
        class_registry: InstanceClassRegistry,
        document: SceneDocument,
    ) -> SceneResult<Self> {
        let mut scene = Self::with_class_registry(class_registry);
        let mut seen_guids = BTreeSet::new();
        let root = scene.add_document_instance(document.root, None, &mut seen_guids)?;
        scene.root = Some(root);
        Ok(scene)
    }

    /// Applies queued structural mutations atomically.
    ///
    /// Mutations are applied to a cloned scene in queue order first. The
    /// original scene is replaced only when every mutation validates and
    /// applies successfully.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] from the first invalid queued mutation without
    /// partially mutating the original scene.
    pub fn apply_mutations(
        &mut self,
        queue: SceneMutationQueue,
    ) -> SceneResult<Vec<SceneMutationResult>> {
        let mut draft = self.clone();
        let mut results = Vec::with_capacity(queue.len());

        for mutation in queue.mutations {
            let result = draft.apply_mutation(mutation)?;
            results.push(result);
        }

        *self = draft;
        Ok(results)
    }

    fn apply_mutation(&mut self, mutation: SceneMutation) -> SceneResult<SceneMutationResult> {
        match mutation {
            SceneMutation::Create {
                parent,
                class_name,
                name,
            } => {
                let id = match parent {
                    Some(parent) => self.add_child(parent, class_name, name)?,
                    None => self.add_root(class_name, name)?,
                };
                Ok(SceneMutationResult::Created { id })
            }
            SceneMutation::Delete { id } => {
                let deleted_ids = self.delete_instance(id)?;
                Ok(SceneMutationResult::Deleted { id, deleted_ids })
            }
            SceneMutation::Rename { id, name } => {
                self.rename_instance(id, name)?;
                Ok(SceneMutationResult::Renamed { id })
            }
            SceneMutation::Reparent { id, new_parent } => {
                let old_parent = self.reparent_instance(id, new_parent)?;
                Ok(SceneMutationResult::Reparented {
                    id,
                    old_parent,
                    new_parent,
                })
            }
        }
    }

    fn create_record(
        &mut self,
        class_name: impl Into<String>,
        name: impl Into<String>,
        parent: Option<InstanceId>,
    ) -> SceneResult<InstanceId> {
        let class_name = class_name.into();
        let name = name.into();
        validate_instance_name(&name)?;
        let properties = default_properties_for_class(self.class_descriptor(&class_name)?, &name)?;

        let id = self.next_instance_id();
        let guid = self.next_instance_guid();
        self.instances.push(InstanceRecord {
            id,
            guid,
            class_name,
            name,
            parent,
            children: Vec::new(),
            properties,
        });
        Ok(id)
    }

    fn delete_instance(&mut self, id: InstanceId) -> SceneResult<Vec<InstanceId>> {
        let index = self.index_of(id)?;
        if Some(id) == self.root {
            return Err(SceneError::CannotDeleteRoot { root_id: id });
        }

        let parent = self.instances[index].parent;
        let mut deleted_ids = Vec::new();
        self.collect_subtree_ids(id, &mut deleted_ids)?;

        if let Some(parent) = parent {
            let parent_index = self.index_of(parent)?;
            self.instances[parent_index]
                .children
                .retain(|child_id| *child_id != id);
        }

        self.instances
            .retain(|instance| !deleted_ids.contains(&instance.id));
        Ok(deleted_ids)
    }

    fn rename_instance(&mut self, id: InstanceId, name: String) -> SceneResult<()> {
        validate_instance_name(&name)?;
        let index = self.index_of(id)?;
        self.instances[index].name.clone_from(&name);
        if let Some(PropertyValue::String(stored_name)) =
            self.instances[index].properties.get_mut("Name")
        {
            *stored_name = name;
        }
        Ok(())
    }

    fn reparent_instance(
        &mut self,
        id: InstanceId,
        new_parent: InstanceId,
    ) -> SceneResult<Option<InstanceId>> {
        let index = self.index_of(id)?;
        self.index_of(new_parent)?;
        if Some(id) == self.root {
            return Err(SceneError::CannotReparentRoot { root_id: id });
        }
        if id == new_parent || self.is_descendant_of(new_parent, id)? {
            return Err(SceneError::ReparentCycle { id, new_parent });
        }

        let old_parent = self.instances[index].parent;
        if old_parent == Some(new_parent) {
            return Ok(old_parent);
        }

        if let Some(old_parent) = old_parent {
            let old_parent_index = self.index_of(old_parent)?;
            self.instances[old_parent_index]
                .children
                .retain(|child_id| *child_id != id);
        }

        let new_parent_index = self.index_of(new_parent)?;
        self.instances[new_parent_index].children.push(id);
        let index = self.index_of(id)?;
        self.instances[index].parent = Some(new_parent);
        Ok(old_parent)
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

    fn collect_subtree_ids(&self, id: InstanceId, output: &mut Vec<InstanceId>) -> SceneResult<()> {
        let instance = self.get(id)?;
        output.push(id);
        for child_id in &instance.children {
            self.collect_subtree_ids(*child_id, output)?;
        }
        Ok(())
    }

    fn is_descendant_of(&self, id: InstanceId, ancestor: InstanceId) -> SceneResult<bool> {
        let mut current = self.get(id)?.parent;
        while let Some(current_id) = current {
            if current_id == ancestor {
                return Ok(true);
            }
            current = self.get(current_id)?.parent;
        }
        Ok(false)
    }

    fn instance_to_document(&self, id: InstanceId) -> SceneResult<SceneInstanceDocument> {
        let instance = self.get(id)?;
        let children = instance
            .children
            .iter()
            .map(|child_id| self.instance_to_document(*child_id))
            .collect::<SceneResult<Vec<_>>>()?;
        Ok(SceneInstanceDocument {
            guid: instance.guid,
            class_name: instance.class_name.clone(),
            name: instance.name.clone(),
            properties: instance.properties.clone(),
            children,
        })
    }

    fn add_document_instance(
        &mut self,
        document: SceneInstanceDocument,
        parent: Option<InstanceId>,
        seen_guids: &mut BTreeSet<InstanceGuid>,
    ) -> SceneResult<InstanceId> {
        if !seen_guids.insert(document.guid) {
            return Err(SceneError::DuplicateInstanceGuid {
                guid: document.guid,
            });
        }

        validate_instance_name(&document.name)?;
        let class_name = document.class_name;
        let name = document.name;
        let guid = document.guid;
        let children = document.children;
        let mut properties =
            default_properties_for_class(self.class_descriptor(&class_name)?, &name)?;
        self.validate_document_properties(&class_name, &document.properties)?;
        for (path, value) in document.properties {
            properties.insert(path, value);
        }
        if let Some(PropertyValue::String(stored_name)) = properties.get_mut("Name") {
            stored_name.clone_from(&name);
        }

        let id = self.next_instance_id();
        self.next_guid = self.next_guid.max(guid.raw() + 1);
        self.instances.push(InstanceRecord {
            id,
            guid,
            class_name,
            name,
            parent,
            children: Vec::new(),
            properties,
        });

        for child in children {
            let child_id = self.add_document_instance(child, Some(id), seen_guids)?;
            let index = self.index_of(id)?;
            self.instances[index].children.push(child_id);
        }

        Ok(id)
    }

    fn validate_document_properties(
        &self,
        class_name: &str,
        properties: &BTreeMap<String, PropertyValue>,
    ) -> SceneResult<()> {
        for (path, value) in properties {
            let descriptor = self.property_descriptor_for_class(class_name, path)?;
            value
                .validate_for_descriptor(descriptor)
                .map_err(|error| property_value_error(class_name, path, error))?;
            if let ("Name", PropertyValue::String(name)) = (path.as_str(), value) {
                validate_instance_name(name)?;
            }
        }
        Ok(())
    }

    fn class_descriptor(&self, class_name: &str) -> SceneResult<&InstanceClassDescriptor> {
        self.class_registry
            .get(class_name)
            .map_err(|_| SceneError::UnknownClass {
                class_name: class_name.to_owned(),
            })
    }

    fn property_descriptor_for_class(
        &self,
        class_name: &str,
        property_path: &str,
    ) -> SceneResult<&PropertyDescriptor> {
        self.class_descriptor(class_name)?
            .property(property_path)
            .ok_or_else(|| SceneError::UnknownProperty {
                class_name: class_name.to_owned(),
                property_path: property_path.to_owned(),
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

fn default_properties_for_class(
    class_descriptor: &InstanceClassDescriptor,
    instance_name: &str,
) -> SceneResult<BTreeMap<String, PropertyValue>> {
    let mut properties = BTreeMap::new();
    for descriptor in &class_descriptor.properties {
        let value = default_property_value(class_descriptor, descriptor, instance_name)?;
        properties.insert(descriptor.path.clone(), value);
    }
    Ok(properties)
}

fn default_property_value(
    class_descriptor: &InstanceClassDescriptor,
    descriptor: &PropertyDescriptor,
    instance_name: &str,
) -> SceneResult<PropertyValue> {
    descriptor.validate().map_err(|error| {
        invalid_property_descriptor(&class_descriptor.class_name, &descriptor.path, error)
    })?;

    if descriptor.path == "Name" && descriptor.value_type == PropertyType::String {
        return Ok(PropertyValue::String(instance_name.to_owned()));
    }

    match &descriptor.default_value {
        PropertyDefault::TypeDefault => {
            PropertyValue::type_default(descriptor.value_type).map_err(|error| {
                property_value_error(&class_descriptor.class_name, &descriptor.path, error)
            })
        }
        PropertyDefault::Value(value) => {
            value.validate_for_descriptor(descriptor).map_err(|error| {
                property_value_error(&class_descriptor.class_name, &descriptor.path, error)
            })?;
            Ok(value.clone())
        }
    }
}

fn property_value_error(class_name: &str, property_path: &str, error: ValueError) -> SceneError {
    match error {
        ValueError::InvalidDescriptor(error) => {
            invalid_property_descriptor(class_name, property_path, error)
        }
        ValueError::NoTypeDefault { value_type } => SceneError::MissingPropertyDefault {
            property_path: property_path.to_owned(),
            value_type,
        },
        ValueError::TypeMismatch {
            path,
            expected,
            actual,
        } => SceneError::PropertyTypeMismatch {
            property_path: path,
            expected,
            actual,
        },
    }
}

fn invalid_property_descriptor(
    class_name: &str,
    property_path: &str,
    error: impl fmt::Display,
) -> SceneError {
    SceneError::InvalidPropertyDescriptor {
        class_name: class_name.to_owned(),
        property_path: property_path.to_owned(),
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene_with_part_class() -> Scene {
        Scene::new()
    }

    #[test]
    fn default_registry_contains_root_services_and_3d_classes_in_order() {
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
                "Folder",
                "Node3D",
                "Part",
                "Camera3D",
                "Light3D",
            ]
        );
        assert_eq!(registry.descriptors().len(), 15);
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
    fn built_in_3d_classes_expose_capabilities_and_shared_properties() {
        let registry = InstanceClassRegistry::with_default_scene_classes().unwrap();

        let folder = registry.get("Folder").unwrap();
        assert!(folder.has_capability(InstanceClassCapability::Container));
        assert!(!folder.has_capability(InstanceClassCapability::Spatial));
        assert!(folder.property("Visible").is_some());

        let part = registry.get("Part").unwrap();
        assert!(part.has_capability(InstanceClassCapability::Spatial));
        assert!(part.has_capability(InstanceClassCapability::Renderable));
        assert_eq!(
            part.property("Transform.Position").unwrap().value_type,
            PropertyType::Vec3
        );
        assert_eq!(
            part.property("Transform.Rotation").unwrap().value_type,
            PropertyType::Quat
        );
        assert_eq!(
            part.property("Transform.Scale").unwrap().value_type,
            PropertyType::Vec3
        );

        let camera = registry.get("Camera3D").unwrap();
        assert!(camera.has_capability(InstanceClassCapability::Camera));

        let light = registry.get("Light3D").unwrap();
        assert!(light.has_capability(InstanceClassCapability::Light));
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
        let mut scene = Scene::new();

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

    #[test]
    fn instance_properties_start_with_descriptor_defaults() {
        let mut scene = scene_with_part_class();
        let part = scene.add_root("Part", "Block").unwrap();

        assert_eq!(
            scene.get_property(part, "Name").unwrap(),
            &PropertyValue::String("Block".to_owned())
        );
        assert_eq!(
            scene.get_property(part, "Visible").unwrap(),
            &PropertyValue::Bool(true)
        );
        assert_eq!(
            scene.get_property(part, "Transform.Position").unwrap(),
            &PropertyValue::Vec3(kinetik_core::Vec3::ZERO)
        );
        assert_eq!(
            scene.get_property(part, "Transform.Rotation").unwrap(),
            &PropertyValue::Quat(kinetik_core::Quat::IDENTITY)
        );
        assert_eq!(
            scene.get_property(part, "Transform.Scale").unwrap(),
            &PropertyValue::Vec3(kinetik_core::Vec3::ONE)
        );
        assert_eq!(
            scene.properties(part).unwrap().keys().collect::<Vec<_>>(),
            vec![
                "Name",
                "Transform.Position",
                "Transform.Rotation",
                "Transform.Scale",
                "Visible"
            ]
        );
    }

    #[test]
    fn set_property_validates_and_stores_values() {
        let mut scene = scene_with_part_class();
        let part = scene.add_root("Part", "Block").unwrap();

        scene
            .set_property(part, "Visible", PropertyValue::Bool(false))
            .unwrap();
        scene
            .set_property(
                part,
                "Transform.Position",
                PropertyValue::Vec3(kinetik_core::Vec3::new(1.0, 2.0, 3.0)),
            )
            .unwrap();
        scene
            .set_property(
                part,
                "Transform.Scale",
                PropertyValue::Vec3(kinetik_core::Vec3::new(2.0, 2.0, 2.0)),
            )
            .unwrap();

        assert_eq!(
            scene.get_property(part, "Visible").unwrap(),
            &PropertyValue::Bool(false)
        );
        assert_eq!(
            scene.get_property(part, "Transform.Position").unwrap(),
            &PropertyValue::Vec3(kinetik_core::Vec3::new(1.0, 2.0, 3.0))
        );
        assert_eq!(
            scene.get_property(part, "Transform.Scale").unwrap(),
            &PropertyValue::Vec3(kinetik_core::Vec3::new(2.0, 2.0, 2.0))
        );
    }

    #[test]
    fn name_property_updates_instance_name_and_path() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();

        scene
            .set_property(
                workspace,
                "Name",
                PropertyValue::String("WorldRoot".to_owned()),
            )
            .unwrap();

        assert_eq!(scene.get(workspace).unwrap().name, "WorldRoot");
        assert_eq!(scene.path(workspace).unwrap(), "/Game/WorldRoot");
        assert_eq!(
            scene.get_property(workspace, "Name").unwrap(),
            &PropertyValue::String("WorldRoot".to_owned())
        );
    }

    #[test]
    fn property_storage_rejects_unknown_and_noncanonical_paths() {
        let mut scene = scene_with_part_class();
        let part = scene.add_root("Part", "Block").unwrap();

        assert_eq!(
            scene.get_property(part, "visible").unwrap_err(),
            SceneError::UnknownProperty {
                class_name: "Part".to_owned(),
                property_path: "visible".to_owned()
            }
        );
        assert_eq!(
            scene
                .set_property(
                    part,
                    "Transform.position",
                    PropertyValue::Vec3(kinetik_core::Vec3::ZERO)
                )
                .unwrap_err(),
            SceneError::UnknownProperty {
                class_name: "Part".to_owned(),
                property_path: "Transform.position".to_owned()
            }
        );
    }

    #[test]
    fn property_storage_rejects_type_mismatches() {
        let mut scene = scene_with_part_class();
        let part = scene.add_root("Part", "Block").unwrap();

        assert_eq!(
            scene
                .set_property(part, "Visible", PropertyValue::String("yes".to_owned()))
                .unwrap_err(),
            SceneError::PropertyTypeMismatch {
                property_path: "Visible".to_owned(),
                expected: PropertyType::Bool,
                actual: PropertyType::String
            }
        );
    }

    #[test]
    fn mutation_queue_applies_valid_batch_in_order() {
        let mut scene = Scene::default_scene().unwrap();
        let game = scene.root_id().unwrap();
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        let audio = scene.get_by_path("/Game/Audio").unwrap().id;

        let mut queue = SceneMutationQueue::new();
        queue.rename(workspace, "World");
        queue.reparent(audio, workspace);
        queue.create_child(workspace, "Workspace", "Zone");

        let results = scene.apply_mutations(queue).unwrap();

        assert_eq!(
            results,
            vec![
                SceneMutationResult::Renamed { id: workspace },
                SceneMutationResult::Reparented {
                    id: audio,
                    old_parent: Some(game),
                    new_parent: workspace
                },
                SceneMutationResult::Created {
                    id: InstanceId::new(11)
                }
            ]
        );
        assert_eq!(scene.path(workspace).unwrap(), "/Game/World");
        assert_eq!(scene.path(audio).unwrap(), "/Game/World/Audio");
        assert_eq!(
            scene.children(workspace).unwrap(),
            &[audio, InstanceId::new(11)]
        );
    }

    #[test]
    fn mutation_queue_deletes_subtrees_deterministically() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
        let parent = scene.add_child(workspace, "Workspace", "Parent").unwrap();
        let child = scene.add_child(parent, "Workspace", "Child").unwrap();

        let mut queue = SceneMutationQueue::new();
        queue.delete(parent);

        assert_eq!(
            scene.apply_mutations(queue).unwrap(),
            vec![SceneMutationResult::Deleted {
                id: parent,
                deleted_ids: vec![parent, child]
            }]
        );
        assert_eq!(scene.children(workspace).unwrap(), &[]);
        assert_eq!(
            scene.get(parent).unwrap_err(),
            SceneError::InvalidInstanceId { id: parent }
        );
        assert_eq!(
            scene.get(child).unwrap_err(),
            SceneError::InvalidInstanceId { id: child }
        );
    }

    #[test]
    fn mutation_queue_rejects_invalid_handles_and_classes() {
        let mut scene = Scene::default_scene().unwrap();
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;

        let mut invalid_parent = SceneMutationQueue::new();
        invalid_parent.create_child(InstanceId::new(99), "Workspace", "MissingParent");
        assert_eq!(
            scene.apply_mutations(invalid_parent).unwrap_err(),
            SceneError::InvalidInstanceId {
                id: InstanceId::new(99)
            }
        );

        let mut unknown_class = SceneMutationQueue::new();
        unknown_class.create_child(workspace, "MissingClass", "Thing");
        assert_eq!(
            scene.apply_mutations(unknown_class).unwrap_err(),
            SceneError::UnknownClass {
                class_name: "MissingClass".to_owned()
            }
        );

        let mut invalid_child = SceneMutationQueue::new();
        invalid_child.reparent(InstanceId::new(99), workspace);
        assert_eq!(
            scene.apply_mutations(invalid_child).unwrap_err(),
            SceneError::InvalidInstanceId {
                id: InstanceId::new(99)
            }
        );
    }

    #[test]
    fn mutation_queue_rejects_root_and_cycle_operations() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
        let child = scene.add_child(workspace, "Workspace", "Child").unwrap();

        let mut delete_root = SceneMutationQueue::new();
        delete_root.delete(game);
        assert_eq!(
            scene.apply_mutations(delete_root).unwrap_err(),
            SceneError::CannotDeleteRoot { root_id: game }
        );

        let mut reparent_root = SceneMutationQueue::new();
        reparent_root.reparent(game, workspace);
        assert_eq!(
            scene.apply_mutations(reparent_root).unwrap_err(),
            SceneError::CannotReparentRoot { root_id: game }
        );

        let mut cycle = SceneMutationQueue::new();
        cycle.reparent(workspace, child);
        assert_eq!(
            scene.apply_mutations(cycle).unwrap_err(),
            SceneError::ReparentCycle {
                id: workspace,
                new_parent: child
            }
        );
    }

    #[test]
    fn failed_mutation_queue_does_not_partially_apply() {
        let mut scene = Scene::new();
        let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
        let child = scene.add_child(workspace, "Workspace", "Child").unwrap();

        let mut queue = SceneMutationQueue::new();
        queue.rename(workspace, "World");
        queue.reparent(workspace, child);

        assert_eq!(
            scene.apply_mutations(queue).unwrap_err(),
            SceneError::ReparentCycle {
                id: workspace,
                new_parent: child
            }
        );
        assert_eq!(scene.get(workspace).unwrap().name, "Workspace");
        assert_eq!(scene.path(child).unwrap(), "/Game/Workspace/Child");
    }

    #[test]
    fn scene_document_captures_default_scene_shape() {
        let scene = Scene::default_scene().unwrap();
        let document = scene.to_document().unwrap();

        assert_eq!(document.root.guid, InstanceGuid::new(1));
        assert_eq!(document.root.class_name, ROOT_CLASS_NAME);
        assert_eq!(document.root.name, "Game");
        assert_eq!(
            document
                .root
                .children
                .iter()
                .map(|child| child.name.as_str())
                .collect::<Vec<_>>(),
            DEFAULT_SERVICE_CLASS_NAMES
        );
    }

    #[test]
    fn scene_document_round_trips_with_deterministic_runtime_ids() {
        let original = Scene::default_scene().unwrap();
        let document = original.to_document().unwrap();
        let restored = Scene::from_document(
            InstanceClassRegistry::with_default_scene_classes().unwrap(),
            document.clone(),
        )
        .unwrap();

        assert_eq!(restored.to_document().unwrap(), document);
        assert_eq!(restored.root_id().unwrap(), InstanceId::new(1));
        assert_eq!(
            restored.get_by_path("/Game/Workspace").unwrap().id,
            InstanceId::new(2)
        );
        assert_eq!(
            restored.get_by_path("/Game/Packages").unwrap().id,
            InstanceId::new(10)
        );
    }

    #[test]
    fn scene_document_properties_are_ordered_by_canonical_path() {
        let mut scene = scene_with_part_class();
        let part = scene.add_root("Part", "Block").unwrap();
        scene
            .set_property(part, "Visible", PropertyValue::Bool(false))
            .unwrap();

        let document = scene.to_document().unwrap();

        assert_eq!(
            document.root.properties.keys().collect::<Vec<_>>(),
            vec![
                "Name",
                "Transform.Position",
                "Transform.Rotation",
                "Transform.Scale",
                "Visible"
            ]
        );
    }

    #[test]
    fn scene_document_rejects_missing_root() {
        let scene = Scene::new();

        assert_eq!(scene.to_document().unwrap_err(), SceneError::MissingRoot);
    }

    #[test]
    fn scene_document_rejects_duplicate_guids() {
        let document = SceneDocument::new(
            SceneInstanceDocument::new(InstanceGuid::new(1), ROOT_CLASS_NAME, "Game")
                .with_children(vec![SceneInstanceDocument::new(
                    InstanceGuid::new(1),
                    "Workspace",
                    "Workspace",
                )]),
        );

        assert_eq!(
            Scene::from_document(
                InstanceClassRegistry::with_default_scene_classes().unwrap(),
                document
            )
            .unwrap_err(),
            SceneError::DuplicateInstanceGuid {
                guid: InstanceGuid::new(1)
            }
        );
    }

    #[test]
    fn scene_document_rejects_unknown_classes_and_invalid_properties() {
        let unknown_class = SceneDocument::new(SceneInstanceDocument::new(
            InstanceGuid::new(1),
            "MissingClass",
            "Game",
        ));
        assert_eq!(
            Scene::from_document(
                InstanceClassRegistry::with_default_scene_classes().unwrap(),
                unknown_class
            )
            .unwrap_err(),
            SceneError::UnknownClass {
                class_name: "MissingClass".to_owned()
            }
        );

        let mut unknown_property = BTreeMap::new();
        unknown_property.insert(
            "Missing".to_owned(),
            PropertyValue::String("value".to_owned()),
        );
        let document = SceneDocument::new(
            SceneInstanceDocument::new(InstanceGuid::new(1), ROOT_CLASS_NAME, "Game")
                .with_properties(unknown_property),
        );
        assert_eq!(
            Scene::from_document(
                InstanceClassRegistry::with_default_scene_classes().unwrap(),
                document
            )
            .unwrap_err(),
            SceneError::UnknownProperty {
                class_name: ROOT_CLASS_NAME.to_owned(),
                property_path: "Missing".to_owned()
            }
        );

        let mut mismatched_property = BTreeMap::new();
        mismatched_property.insert("Name".to_owned(), PropertyValue::Bool(true));
        let document = SceneDocument::new(
            SceneInstanceDocument::new(InstanceGuid::new(1), ROOT_CLASS_NAME, "Game")
                .with_properties(mismatched_property),
        );
        assert_eq!(
            Scene::from_document(
                InstanceClassRegistry::with_default_scene_classes().unwrap(),
                document
            )
            .unwrap_err(),
            SceneError::PropertyTypeMismatch {
                property_path: "Name".to_owned(),
                expected: PropertyType::String,
                actual: PropertyType::Bool
            }
        );
    }

    #[test]
    fn default_scene_has_exact_adr_0002_hierarchy() {
        let scene = Scene::default_scene().unwrap();
        let root_id = scene.root_id().unwrap();
        let root = scene.get(root_id).unwrap();

        assert_eq!(root.name, "Game");
        assert_eq!(root.class_name, ROOT_CLASS_NAME);
        assert_eq!(scene.path(root_id).unwrap(), "/Game");

        let service_names: Vec<&str> = scene
            .children(root_id)
            .unwrap()
            .iter()
            .map(|id| scene.get(*id).unwrap().name.as_str())
            .collect();
        assert_eq!(service_names, DEFAULT_SERVICE_CLASS_NAMES);
    }

    #[test]
    fn default_scene_services_are_visible_by_path() {
        let scene = Scene::default_scene().unwrap();

        for class_name in DEFAULT_SERVICE_CLASS_NAMES {
            let path = format!("/Game/{class_name}");
            let service = scene.get_by_path(&path).unwrap();
            assert_eq!(service.name, class_name);
            assert_eq!(service.class_name, class_name);
        }
    }

    #[test]
    fn default_scene_ids_are_deterministic() {
        let first = Scene::default_scene().unwrap();
        let second = Scene::default_scene().unwrap();

        assert_eq!(first.root_id().unwrap().raw(), 1);
        assert_eq!(second.root_id().unwrap().raw(), 1);

        for (index, class_name) in DEFAULT_SERVICE_CLASS_NAMES.iter().enumerate() {
            let expected_raw = index as u64 + 2;
            let path = format!("/Game/{class_name}");
            let first_service = first.get_by_path(&path).unwrap();
            let second_service = second.get_by_path(&path).unwrap();

            assert_eq!(first_service.id.raw(), expected_raw);
            assert_eq!(first_service.guid.raw(), expected_raw);
            assert_eq!(second_service.id.raw(), expected_raw);
            assert_eq!(second_service.guid.raw(), expected_raw);
        }
    }
}
