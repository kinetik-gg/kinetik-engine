use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticLocation, DiagnosticSeverity,
    DiagnosticSource,
};

use crate::{ScriptAssetRef, ScriptAttachmentTarget};

/// Stable diagnostic codes owned by the script contract.
pub struct ScriptDiagnosticCode;

impl ScriptDiagnosticCode {
    /// A referenced script asset could not be resolved.
    pub const MISSING_SCRIPT: DiagnosticCode = DiagnosticCode::new("KT_SCRIPT_MISSING");

    /// A script-facing instance or resource handle is no longer valid.
    pub const INVALID_HANDLE: DiagnosticCode = DiagnosticCode::new("KT_SCRIPT_INVALID_HANDLE");
}

/// Script diagnostic provenance shared by runtime, editor, HTTP, and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptDiagnosticContext {
    /// Script asset associated with the diagnostic.
    pub script: ScriptAssetRef,
    /// Owning instance target when known.
    pub target: Option<ScriptAttachmentTarget>,
    /// Scene path when known.
    pub scene_path: Option<String>,
}

impl ScriptDiagnosticContext {
    /// Creates diagnostic provenance for a script asset.
    #[must_use]
    pub const fn new(script: ScriptAssetRef) -> Self {
        Self {
            script,
            target: None,
            scene_path: None,
        }
    }

    /// Adds owning instance provenance.
    #[must_use]
    pub const fn with_target(mut self, target: ScriptAttachmentTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Adds scene path provenance.
    #[must_use]
    pub fn with_scene_path(mut self, scene_path: impl Into<String>) -> Self {
        self.scene_path = Some(scene_path.into());
        self
    }

    fn location(&self) -> DiagnosticLocation {
        let mut location = DiagnosticLocation::new();
        location.asset_path = Some(self.script.path.clone());
        location.script_path = Some(self.script.path.clone());
        location.source_range = self.script.source_range;
        location.scene_path.clone_from(&self.scene_path);

        if let Some(target) = self.target {
            location.instance_guid = target.instance_guid;
        }

        location
    }
}

/// Creates a missing-script diagnostic with script and instance provenance.
#[must_use]
pub fn missing_script_diagnostic(context: &ScriptDiagnosticContext) -> Diagnostic {
    Diagnostic::new(
        ScriptDiagnosticCode::MISSING_SCRIPT,
        DiagnosticSeverity::Error,
        DiagnosticSource::new("Script"),
        format!(
            "Script asset '{}' could not be resolved.",
            context.script.path
        ),
    )
    .with_blocking_scope(DiagnosticBlockingScope::Play)
    .with_location(context.location())
    .with_suggested_fix("Restore the script asset or remove the script attachment.")
}

/// Creates an invalid-handle diagnostic with script and instance provenance.
#[must_use]
pub fn invalid_script_handle_diagnostic(context: &ScriptDiagnosticContext) -> Diagnostic {
    Diagnostic::new(
        ScriptDiagnosticCode::INVALID_HANDLE,
        DiagnosticSeverity::Error,
        DiagnosticSource::new("Script"),
        "Script attempted to use an invalidated engine handle.",
    )
    .with_blocking_scope(DiagnosticBlockingScope::Play)
    .with_location(context.location())
}
