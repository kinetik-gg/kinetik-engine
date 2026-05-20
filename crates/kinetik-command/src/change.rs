use kinetik_core::InstanceGuid;
use kinetik_reflect::PropertyValue;

use crate::{
    validate_command_kind, validate_dirty_summary, CommandModelResult, CommandTargetMode,
    UndoGroupId,
};

/// Semantic command target affected by a change record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeTarget {
    /// Scene instance target.
    Instance {
        /// Stable edit-world instance GUID when known.
        guid: Option<InstanceGuid>,
        /// Human-readable scene path when known.
        scene_path: Option<String>,
    },
    /// Reflected property target.
    Property {
        /// Stable edit-world instance GUID when known.
        instance_guid: Option<InstanceGuid>,
        /// Human-readable scene path when known.
        scene_path: Option<String>,
        /// Canonical reflected property path.
        property_path: String,
    },
    /// Source asset target.
    Asset {
        /// Stable asset GUID string when known.
        asset_guid: Option<String>,
        /// Project resource path such as `res://assets/tree.glb`.
        asset_path: String,
    },
    /// Script source target.
    Script {
        /// Workspace-relative script path.
        script_path: String,
    },
    /// Source document target.
    Document {
        /// Workspace-relative source document path.
        document_path: String,
    },
}

/// Reflected property value transition recorded by a semantic change.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyValueChange {
    /// Canonical reflected property path.
    pub property_path: String,
    /// Value before the command when known.
    pub old_value: Option<PropertyValue>,
    /// Value after the command when known.
    pub new_value: Option<PropertyValue>,
}

impl PropertyValueChange {
    /// Creates a reflected property value transition.
    #[must_use]
    pub fn new(
        property_path: impl Into<String>,
        old_value: Option<PropertyValue>,
        new_value: Option<PropertyValue>,
    ) -> Self {
        Self {
            property_path: property_path.into(),
            old_value,
            new_value,
        }
    }
}

/// Semantic change record produced by a successful command.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandChangeRecord {
    command_kind: String,
    target_mode: CommandTargetMode,
    targets: Vec<ChangeTarget>,
    property_value_change: Option<PropertyValueChange>,
    affected_documents: Vec<String>,
    undo_group: Option<UndoGroupId>,
    dirty_summary: String,
}

impl CommandChangeRecord {
    /// Creates a semantic change record.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CommandError`] when required text fields are empty.
    pub fn new(
        command_kind: impl Into<String>,
        target_mode: CommandTargetMode,
        dirty_summary: impl Into<String>,
    ) -> CommandModelResult<Self> {
        Ok(Self {
            command_kind: validate_command_kind(command_kind.into())?,
            target_mode,
            targets: Vec::new(),
            property_value_change: None,
            affected_documents: Vec::new(),
            undo_group: None,
            dirty_summary: validate_dirty_summary(dirty_summary.into())?,
        })
    }

    /// Adds affected semantic targets in deterministic order.
    #[must_use]
    pub fn with_targets(mut self, targets: Vec<ChangeTarget>) -> Self {
        self.targets = targets;
        self
    }

    /// Adds a reflected property value transition.
    #[must_use]
    pub fn with_property_value_change(mut self, change: PropertyValueChange) -> Self {
        self.property_value_change = Some(change);
        self
    }

    /// Adds affected source documents in deterministic order.
    #[must_use]
    pub fn with_affected_documents(mut self, affected_documents: Vec<String>) -> Self {
        self.affected_documents = affected_documents;
        self
    }

    /// Assigns this change record to an undo group.
    #[must_use]
    pub const fn with_undo_group(mut self, undo_group: UndoGroupId) -> Self {
        self.undo_group = Some(undo_group);
        self
    }

    /// Returns the command kind that produced this record.
    #[must_use]
    pub fn command_kind(&self) -> &str {
        &self.command_kind
    }

    /// Returns the target mode for this semantic change.
    #[must_use]
    pub const fn target_mode(&self) -> CommandTargetMode {
        self.target_mode
    }

    /// Returns affected semantic targets in deterministic order.
    #[must_use]
    pub fn targets(&self) -> &[ChangeTarget] {
        &self.targets
    }

    /// Returns the reflected property value transition when this record has one.
    #[must_use]
    pub const fn property_value_change(&self) -> Option<&PropertyValueChange> {
        self.property_value_change.as_ref()
    }

    /// Returns affected source documents in deterministic order.
    #[must_use]
    pub fn affected_documents(&self) -> &[String] {
        &self.affected_documents
    }

    /// Returns the undo group for this change when present.
    #[must_use]
    pub const fn undo_group(&self) -> Option<UndoGroupId> {
        self.undo_group
    }

    /// Returns human-readable dirty-state summary text.
    #[must_use]
    pub fn dirty_summary(&self) -> &str {
        &self.dirty_summary
    }
}
