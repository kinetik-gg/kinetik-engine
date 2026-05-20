use kinetik_core::{InstanceGuid, InstanceId, ResourceId, ScriptId, SourceRange};

/// Language backend for a script asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScriptLanguage {
    name: String,
}

impl ScriptLanguage {
    /// Creates a script language identifier.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the accepted Luau language identifier.
    #[must_use]
    pub fn luau() -> Self {
        Self::new("luau")
    }

    /// Returns the language name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

/// Reference to a project-owned script asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptAssetRef {
    /// Runtime resource handle when the asset is loaded.
    pub resource_id: Option<ResourceId>,
    /// Stable project path, such as `res://scripts/player.luau`.
    pub path: String,
    /// Language backend responsible for the asset.
    pub language: ScriptLanguage,
    /// Optional source range for diagnostics or permission provenance.
    pub source_range: Option<SourceRange>,
}

impl ScriptAssetRef {
    /// Creates a script asset reference.
    #[must_use]
    pub fn new(path: impl Into<String>, language: ScriptLanguage) -> Self {
        Self {
            resource_id: None,
            path: path.into(),
            language,
            source_range: None,
        }
    }

    /// Attaches a loaded runtime resource handle.
    #[must_use]
    pub const fn with_resource_id(mut self, resource_id: ResourceId) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    /// Attaches a source range.
    #[must_use]
    pub const fn with_source_range(mut self, source_range: SourceRange) -> Self {
        self.source_range = Some(source_range);
        self
    }
}

/// Runtime attachment ID for a script bound to an instance.
pub type ScriptAttachmentId = ScriptId;

/// Instance targeted by a script attachment.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScriptAttachmentTarget {
    /// Runtime instance handle in the play world.
    pub instance_id: InstanceId,
    /// Stable edit GUID when this runtime instance derives from saved state.
    pub instance_guid: Option<InstanceGuid>,
}

impl ScriptAttachmentTarget {
    /// Creates a target from a runtime instance ID.
    #[must_use]
    pub const fn runtime_only(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            instance_guid: None,
        }
    }

    /// Creates a target with edit GUID provenance.
    #[must_use]
    pub const fn with_guid(instance_id: InstanceId, instance_guid: InstanceGuid) -> Self {
        Self {
            instance_id,
            instance_guid: Some(instance_guid),
        }
    }
}

/// Binding between a script asset and an instance in a runtime world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptAttachment {
    /// Runtime attachment ID.
    pub id: ScriptAttachmentId,
    /// Project-owned script asset.
    pub script: ScriptAssetRef,
    /// Instance the script is attached to.
    pub target: ScriptAttachmentTarget,
    /// Scene path at the time the attachment was created.
    pub scene_path: Option<String>,
}

impl ScriptAttachment {
    /// Creates a script attachment contract.
    #[must_use]
    pub const fn new(
        id: ScriptAttachmentId,
        script: ScriptAssetRef,
        target: ScriptAttachmentTarget,
    ) -> Self {
        Self {
            id,
            script,
            target,
            scene_path: None,
        }
    }

    /// Adds scene path provenance.
    #[must_use]
    pub fn with_scene_path(mut self, scene_path: impl Into<String>) -> Self {
        self.scene_path = Some(scene_path.into());
        self
    }
}
