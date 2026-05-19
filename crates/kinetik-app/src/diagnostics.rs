use kinetik_core::{Diagnostic, DiagnosticSource, InstanceGuid};

use crate::runtime_world::{RuntimeInstanceId, RuntimeWorldId};

/// Runtime provenance attached to diagnostics and chronological log records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeAttribution {
    /// Runtime world that produced the record.
    pub world_id: RuntimeWorldId,
    /// Runtime frame index when known.
    pub frame_index: Option<u64>,
    /// Fixed-step index when the record was produced inside fixed simulation.
    pub fixed_step_index: Option<u64>,
    /// Runtime subsystem that produced the record.
    pub source: DiagnosticSource,
    /// Runtime instance associated with the record when known.
    pub instance_id: Option<RuntimeInstanceId>,
    /// Saved edit GUID when the runtime instance derives from edit state.
    pub edit_guid: Option<InstanceGuid>,
    /// Script asset path associated with the record when known.
    pub script_path: Option<String>,
}

impl RuntimeAttribution {
    /// Creates runtime attribution for a world and source subsystem.
    #[must_use]
    pub const fn new(world_id: RuntimeWorldId, source: DiagnosticSource) -> Self {
        Self {
            world_id,
            frame_index: None,
            fixed_step_index: None,
            source,
            instance_id: None,
            edit_guid: None,
            script_path: None,
        }
    }

    /// Sets the runtime frame index.
    #[must_use]
    pub const fn with_frame_index(mut self, frame_index: u64) -> Self {
        self.frame_index = Some(frame_index);
        self
    }

    /// Sets the fixed-step index.
    #[must_use]
    pub const fn with_fixed_step_index(mut self, fixed_step_index: u64) -> Self {
        self.fixed_step_index = Some(fixed_step_index);
        self
    }

    /// Sets the runtime instance and optional saved edit GUID.
    #[must_use]
    pub const fn with_instance(
        mut self,
        instance_id: RuntimeInstanceId,
        edit_guid: Option<InstanceGuid>,
    ) -> Self {
        self.instance_id = Some(instance_id);
        self.edit_guid = edit_guid;
        self
    }

    /// Sets the script asset path associated with the record.
    #[must_use]
    pub fn with_script_path(mut self, script_path: impl Into<String>) -> Self {
        self.script_path = Some(script_path.into());
        self
    }
}

/// Runtime diagnostic with world/frame attribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDiagnosticRecord {
    /// Runtime attribution for this diagnostic.
    pub attribution: RuntimeAttribution,
    /// Structured diagnostic payload.
    pub diagnostic: Diagnostic,
}

impl RuntimeDiagnosticRecord {
    /// Creates a runtime-attributed diagnostic record.
    #[must_use]
    pub const fn new(diagnostic: Diagnostic, attribution: RuntimeAttribution) -> Self {
        Self {
            attribution,
            diagnostic,
        }
    }
}

/// Runtime chronological log severity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuntimeLogLevel {
    /// Detailed diagnostic information for developers.
    Debug,
    /// Informational runtime event.
    Info,
    /// Suspicious runtime event that does not immediately invalidate play.
    Warning,
    /// Runtime failure that may block play or a subsystem.
    Error,
}

/// Runtime log message with world/frame attribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLogRecord {
    /// Runtime attribution for this log record.
    pub attribution: RuntimeAttribution,
    /// Runtime log severity.
    pub level: RuntimeLogLevel,
    /// Human-readable log message.
    pub message: String,
}

impl RuntimeLogRecord {
    /// Creates a runtime-attributed log record.
    #[must_use]
    pub fn new(
        level: RuntimeLogLevel,
        attribution: RuntimeAttribution,
        message: impl Into<String>,
    ) -> Self {
        Self {
            attribution,
            level,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetik_core::{DiagnosticCode, DiagnosticSeverity};

    #[test]
    fn attribution_tracks_world_frame_fixed_step_and_source() {
        let attribution =
            RuntimeAttribution::new(RuntimeWorldId::new(7), DiagnosticSource::new("Script"))
                .with_frame_index(12)
                .with_fixed_step_index(4);

        assert_eq!(attribution.world_id, RuntimeWorldId::new(7));
        assert_eq!(attribution.frame_index, Some(12));
        assert_eq!(attribution.fixed_step_index, Some(4));
        assert_eq!(attribution.source.as_str(), "Script");
        assert_eq!(attribution.instance_id, None);
    }

    #[test]
    fn attribution_tracks_runtime_instance_edit_guid_and_script_path() {
        let attribution =
            RuntimeAttribution::new(RuntimeWorldId::new(1), DiagnosticSource::new("Luau"))
                .with_instance(RuntimeInstanceId::new(3), Some(InstanceGuid::new(99)))
                .with_script_path("res://scripts/player.luau");

        assert_eq!(attribution.instance_id, Some(RuntimeInstanceId::new(3)));
        assert_eq!(attribution.edit_guid, Some(InstanceGuid::new(99)));
        assert_eq!(
            attribution.script_path.as_deref(),
            Some("res://scripts/player.luau")
        );
    }

    #[test]
    fn diagnostic_record_preserves_payload_and_attribution() {
        let attribution =
            RuntimeAttribution::new(RuntimeWorldId::new(2), DiagnosticSource::new("Physics"))
                .with_frame_index(5);
        let diagnostic = Diagnostic::new(
            DiagnosticCode::new("KT_TEST_RUNTIME"),
            DiagnosticSeverity::Warning,
            attribution.source,
            "Runtime warning",
        );

        let record = RuntimeDiagnosticRecord::new(diagnostic.clone(), attribution.clone());

        assert_eq!(record.attribution, attribution);
        assert_eq!(record.diagnostic, diagnostic);
    }

    #[test]
    fn log_record_preserves_level_message_and_attribution() {
        let attribution =
            RuntimeAttribution::new(RuntimeWorldId::new(2), DiagnosticSource::new("Runtime"))
                .with_frame_index(9);

        let record =
            RuntimeLogRecord::new(RuntimeLogLevel::Info, attribution.clone(), "Started play");

        assert_eq!(record.attribution, attribution);
        assert_eq!(record.level, RuntimeLogLevel::Info);
        assert_eq!(record.message, "Started play");
    }
}
