//! Inspector property projection for selected scene instances.

use kinetik_command::{set_scene_instance_property, CommandError};
use kinetik_core::{InstanceGuid, InstanceId};
use kinetik_reflect::{
    EditorEditability, EditorHint, PropertyType, PropertyValue, SerializationPolicy,
};
use kinetik_scene::Scene;

use crate::{EditorDocumentSelection, EditorSession};

/// One reflected property row shown by the Inspector.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorPropertyRow {
    /// Canonical reflected property path.
    pub path: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Reflected property type.
    pub value_type: PropertyType,
    /// Current property value.
    pub value: PropertyValue,
    /// Whether the property is editor-editable.
    pub editability: EditorEditability,
    /// Optional read-only reason.
    pub read_only_reason: Option<String>,
    /// Inspector presentation hint from reflection metadata.
    pub editor_hint: EditorHint,
    /// Serialization policy from reflection metadata.
    pub serialization: SerializationPolicy,
}

/// Inspector snapshot for one scene instance.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorSnapshot {
    /// Target instance ID.
    pub instance_id: InstanceId,
    /// Target instance GUID.
    pub instance_guid: InstanceGuid,
    /// Target scene path.
    pub scene_path: String,
    /// Target class name.
    pub class_name: String,
    rows: Vec<InspectorPropertyRow>,
}

impl InspectorSnapshot {
    /// Builds an Inspector snapshot from reflection metadata and current values.
    ///
    /// # Errors
    ///
    /// Returns an error message when the instance or class descriptor is invalid.
    pub fn from_scene_instance(scene: &Scene, instance_id: InstanceId) -> Result<Self, String> {
        let instance = scene.get(instance_id).map_err(|error| error.to_string())?;
        let class = scene
            .class_registry()
            .get(&instance.class_name)
            .map_err(|error| error.to_string())?;
        let mut rows = Vec::with_capacity(class.properties.len());

        for descriptor in &class.properties {
            let value = scene
                .get_property(instance_id, &descriptor.path)
                .map_err(|error| error.to_string())?
                .clone();
            rows.push(InspectorPropertyRow {
                path: descriptor.path.clone(),
                display_name: descriptor.display_name.clone(),
                value_type: descriptor.value_type,
                value,
                editability: descriptor.editor_editability,
                read_only_reason: descriptor.read_only_reason.clone(),
                editor_hint: descriptor.editor_hint.clone(),
                serialization: descriptor.serialization,
            });
        }

        Ok(Self {
            instance_id,
            instance_guid: instance.guid,
            scene_path: scene.path(instance_id).map_err(|error| error.to_string())?,
            class_name: instance.class_name.clone(),
            rows,
        })
    }

    /// Returns reflected property rows in descriptor order.
    #[must_use]
    pub fn rows(&self) -> &[InspectorPropertyRow] {
        &self.rows
    }

    /// Finds a property row by canonical property path.
    #[must_use]
    pub fn row(&self, path: &str) -> Option<&InspectorPropertyRow> {
        self.rows.iter().find(|row| row.path == path)
    }
}

/// Error returned by command-backed Inspector actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InspectorCommandError {
    /// No project/scene is open in the editor session.
    NoActiveScene,
    /// No scene instance is selected.
    NoSelectedInstance,
    /// Existing command validation rejected the action.
    Command(CommandError),
    /// Scene/reflection inspection failed.
    Scene(String),
}

impl std::fmt::Display for InspectorCommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveScene => formatter.write_str("no active scene is open"),
            Self::NoSelectedInstance => formatter.write_str("no scene instance is selected"),
            Self::Command(error) => write!(formatter, "{error}"),
            Self::Scene(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for InspectorCommandError {}

impl From<CommandError> for InspectorCommandError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

impl EditorSession {
    /// Returns an Inspector snapshot for `instance_id`.
    ///
    /// # Errors
    ///
    /// Returns [`InspectorCommandError`] when no scene is open or reflection
    /// projection fails.
    pub fn inspector_snapshot(
        &self,
        instance_id: InstanceId,
    ) -> Result<InspectorSnapshot, InspectorCommandError> {
        let scene = self
            .active_scene
            .as_ref()
            .ok_or(InspectorCommandError::NoActiveScene)?;
        InspectorSnapshot::from_scene_instance(scene, instance_id)
            .map_err(InspectorCommandError::Scene)
    }

    /// Returns an Inspector snapshot for the selected scene instance.
    ///
    /// # Errors
    ///
    /// Returns [`InspectorCommandError`] when there is no selected scene
    /// instance or reflection projection fails.
    pub fn selected_inspector_snapshot(&self) -> Result<InspectorSnapshot, InspectorCommandError> {
        let EditorDocumentSelection::SceneInstance { id, .. } = self.selection.document() else {
            return Err(InspectorCommandError::NoSelectedInstance);
        };
        self.inspector_snapshot(*id)
    }

    /// Sets a reflected property through the shared scene property command path.
    ///
    /// # Errors
    ///
    /// Returns [`InspectorCommandError`] when no scene is open, validation
    /// fails, or selection cannot be refreshed after mutation.
    pub fn inspector_set_property(
        &mut self,
        instance_id: InstanceId,
        property_path: impl Into<String>,
        value: PropertyValue,
    ) -> Result<(), InspectorCommandError> {
        let document_path = self
            .active_scene_document_path()
            .map_err(|_| InspectorCommandError::NoActiveScene)?;
        let result = set_scene_instance_property(
            self.active_scene
                .as_mut()
                .ok_or(InspectorCommandError::NoActiveScene)?,
            instance_id,
            property_path,
            value,
            document_path,
        )?;
        self.command_history
            .commit_result("Set Property", &result.command)
            .map_err(InspectorCommandError::Command)?;
        self.refresh_selection_if_selected(instance_id)
            .map_err(|error| InspectorCommandError::Scene(error.to_string()))?;
        Ok(())
    }

    /// Sets a reflected property on the selected scene instance.
    ///
    /// # Errors
    ///
    /// Returns [`InspectorCommandError`] when no scene instance is selected or
    /// the property edit fails validation.
    pub fn inspector_set_selected_property(
        &mut self,
        property_path: impl Into<String>,
        value: PropertyValue,
    ) -> Result<(), InspectorCommandError> {
        let EditorDocumentSelection::SceneInstance { id, .. } = self.selection.document() else {
            return Err(InspectorCommandError::NoSelectedInstance);
        };
        self.inspector_set_property(*id, property_path, value)
    }
}
