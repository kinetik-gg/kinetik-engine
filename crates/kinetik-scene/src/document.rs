use std::collections::{BTreeMap, BTreeSet};

use kinetik_core::{InstanceGuid, InstanceId};
use kinetik_reflect::PropertyValue;

use crate::scene::{default_properties_for_class, property_value_error, validate_instance_name};
use crate::{InstanceClassRegistry, InstanceRecord, Scene, SceneError, SceneResult};

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

impl Scene {
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
}
