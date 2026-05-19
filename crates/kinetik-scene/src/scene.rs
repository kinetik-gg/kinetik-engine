use std::collections::BTreeMap;
use std::fmt;

use kinetik_core::{InstanceGuid, InstanceId};
use kinetik_reflect::{
    PropertyDefault, PropertyDescriptor, PropertyType, PropertyValue, ValueError,
};

use crate::transform::is_transform_property_path;
use crate::{
    InstanceClassCapability, InstanceClassDescriptor, InstanceClassRegistry, InstanceRecord,
    DEFAULT_SERVICE_CLASS_NAMES, ROOT_CLASS_NAME,
};
use crate::{SceneError, SceneResult};

/// In-memory scene hierarchy.
#[derive(Debug, Clone)]
pub struct Scene {
    pub(crate) class_registry: InstanceClassRegistry,
    pub(crate) instances: Vec<InstanceRecord>,
    pub(crate) root: Option<InstanceId>,
    pub(crate) next_id: u64,
    pub(crate) next_guid: u64,
    pub(crate) transform_revision: u64,
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
            transform_revision: 0,
        }
    }

    /// Returns the scene class registry.
    #[must_use]
    pub const fn class_registry(&self) -> &InstanceClassRegistry {
        &self.class_registry
    }

    /// Returns the current scene transform revision.
    ///
    /// The revision advances when hierarchy or transform-property edits can
    /// affect local or world transform results.
    #[must_use]
    pub const fn transform_revision(&self) -> u64 {
        self.transform_revision
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

        if is_transform_property_path(property_path) {
            self.advance_transform_revision();
        }

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
        self.advance_transform_revision();
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
    pub(crate) fn class_descriptor(
        &self,
        class_name: &str,
    ) -> SceneResult<&InstanceClassDescriptor> {
        self.class_registry
            .get(class_name)
            .map_err(|_| SceneError::UnknownClass {
                class_name: class_name.to_owned(),
            })
    }

    pub(crate) fn property_descriptor_for_class(
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

    pub(crate) fn is_spatial_instance(&self, instance: &InstanceRecord) -> SceneResult<bool> {
        Ok(self
            .class_descriptor(&instance.class_name)?
            .has_capability(InstanceClassCapability::Spatial))
    }

    pub(crate) fn require_spatial_instance(&self, instance: &InstanceRecord) -> SceneResult<()> {
        if self.is_spatial_instance(instance)? {
            return Ok(());
        }
        Err(SceneError::NonSpatialInstance {
            id: instance.id,
            class_name: instance.class_name.clone(),
        })
    }

    pub(crate) fn index_of(&self, id: InstanceId) -> SceneResult<usize> {
        self.instances
            .iter()
            .position(|instance| instance.id == id)
            .ok_or(SceneError::InvalidInstanceId { id })
    }

    pub(crate) fn next_instance_id(&mut self) -> InstanceId {
        let id = InstanceId::new(self.next_id);
        self.next_id += 1;
        id
    }

    fn next_instance_guid(&mut self) -> InstanceGuid {
        let guid = InstanceGuid::new(self.next_guid);
        self.next_guid += 1;
        guid
    }

    pub(crate) fn advance_transform_revision(&mut self) {
        self.transform_revision = self
            .transform_revision
            .checked_add(1)
            .expect("scene transform revision exhausted u64");
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
pub(crate) fn validate_instance_name(name: &str) -> SceneResult<()> {
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

pub(crate) fn default_properties_for_class(
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

pub(crate) fn property_value_error(
    class_name: &str,
    property_path: &str,
    error: ValueError,
) -> SceneError {
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
