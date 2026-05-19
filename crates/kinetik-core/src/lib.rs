//! Core primitives for Kinetik Engine.
//!
//! This crate contains small shared foundation types used by higher-level crates.

use core::{fmt, num::NonZeroU64};

/// Standard result type used by Kinetik crates.
pub type KinetikResult<T> = Result<T, KinetikError>;

/// Stable diagnostic code used by tests, tools, editor panels, and agents.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    /// `KinetikError` code for an invalid typed handle.
    pub const CORE_INVALID_HANDLE: Self = Self("KT_CORE_INVALID_HANDLE");

    /// `KinetikError` code for a missing item.
    pub const CORE_NOT_FOUND: Self = Self("KT_CORE_NOT_FOUND");

    /// `KinetikError` code for a feature that is not implemented yet.
    pub const CORE_NOT_IMPLEMENTED: Self = Self("KT_CORE_NOT_IMPLEMENTED");

    /// Creates a diagnostic code from a stable string.
    #[must_use]
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    /// Returns the stable diagnostic code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// System that produced a diagnostic.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticSource(&'static str);

impl DiagnosticSource {
    /// Core foundation source used for core-owned errors and diagnostics.
    pub const CORE: Self = Self("Core");

    /// Creates a diagnostic source from a stable source name.
    #[must_use]
    pub const fn new(source: &'static str) -> Self {
        Self(source)
    }

    /// Returns the stable source name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for DiagnosticSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Severity of a current project or runtime health diagnostic.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    /// Useful state that does not require action.
    Info,
    /// Suspicious state that may become a problem.
    Warning,
    /// Invalid state that blocks at least one workflow.
    Error,
    /// Unrecoverable state that prevents safe continuation.
    Fatal,
}

/// Workflow blocked by a diagnostic.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticBlockingScope {
    /// Blocks editing.
    Edit,
    /// Blocks saving.
    Save,
    /// Blocks play mode.
    Play,
    /// Blocks import.
    Import,
    /// Blocks builds.
    Build,
    /// Blocks bundle generation.
    Bundle,
    /// Blocks publishing.
    Publish,
    /// Blocks tests.
    Test,
}

/// Source text range associated with a diagnostic.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceRange {
    /// One-based starting line.
    pub start_line: u32,
    /// One-based starting column.
    pub start_column: u32,
    /// One-based ending line.
    pub end_line: u32,
    /// One-based ending column.
    pub end_column: u32,
}

impl SourceRange {
    /// Creates a source range from one-based line and column values.
    #[must_use]
    pub const fn new(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }
}

/// Optional target details for a diagnostic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticLocation {
    /// Related instance GUID when applicable.
    pub instance_guid: Option<InstanceGuid>,
    /// Related scene path when applicable.
    pub scene_path: Option<String>,
    /// Related asset project path when applicable.
    pub asset_path: Option<String>,
    /// Related script path when applicable.
    pub script_path: Option<String>,
    /// Related script or source range when applicable.
    pub source_range: Option<SourceRange>,
    /// Related reflected property path when applicable.
    pub property_path: Option<String>,
}

impl DiagnosticLocation {
    /// Creates an empty diagnostic location.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Whether an automated agent may attempt a diagnostic repair.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AgentRepair {
    /// Automated repair is not allowed without higher-level approval.
    #[default]
    NotAllowed,
    /// Automated repair is allowed through validated commands.
    Allowed,
}

/// Structured current health record for editor, runtime, build, and agent workflows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable code for tests and tools.
    pub code: DiagnosticCode,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// Human-readable diagnostic message.
    pub message: String,
    /// System that produced the diagnostic.
    pub source: DiagnosticSource,
    /// Workflow blocked by this diagnostic when known.
    pub blocking: Option<DiagnosticBlockingScope>,
    /// Optional location details.
    pub location: DiagnosticLocation,
    /// Safe suggested fix when available.
    pub suggested_fix: Option<String>,
    /// Whether an automated agent may attempt repair.
    pub agent_repair: AgentRepair,
}

impl Diagnostic {
    /// Creates a diagnostic with no location, suggested fix, or agent repair permission.
    #[must_use]
    pub fn new(
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        source: DiagnosticSource,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            source,
            blocking: None,
            location: DiagnosticLocation::default(),
            suggested_fix: None,
            agent_repair: AgentRepair::NotAllowed,
        }
    }

    /// Sets the blocking workflow scope.
    #[must_use]
    pub const fn with_blocking_scope(mut self, blocking: DiagnosticBlockingScope) -> Self {
        self.blocking = Some(blocking);
        self
    }

    /// Sets the diagnostic location.
    #[must_use]
    pub fn with_location(mut self, location: DiagnosticLocation) -> Self {
        self.location = location;
        self
    }

    /// Sets a safe suggested fix.
    #[must_use]
    pub fn with_suggested_fix(mut self, suggested_fix: impl Into<String>) -> Self {
        self.suggested_fix = Some(suggested_fix.into());
        self
    }

    /// Marks the diagnostic as repairable by automated agents through validated commands.
    #[must_use]
    pub const fn allow_agent_repair(mut self) -> Self {
        self.agent_repair = AgentRepair::Allowed;
        self
    }
}

/// Foundational engine error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KinetikError {
    /// A handle or ID was not valid in the receiving system.
    InvalidHandle {
        /// Human-readable handle kind, such as `InstanceId`.
        kind: &'static str,
        /// Raw handle value that failed validation.
        id: u64,
    },
    /// A requested item was not found.
    NotFound {
        /// Human-readable item kind, such as `Resource`.
        kind: &'static str,
        /// Requested item name or path.
        name: String,
    },
    /// The operation is not implemented yet.
    NotImplemented {
        /// Feature name that is not implemented yet.
        feature: &'static str,
    },
}

impl fmt::Display for KinetikError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle { kind, id } => write!(f, "invalid {kind} handle: {id}"),
            Self::NotFound { kind, name } => write!(f, "{kind} not found: {name}"),
            Self::NotImplemented { feature } => write!(f, "feature not implemented: {feature}"),
        }
    }
}

impl std::error::Error for KinetikError {}

impl KinetikError {
    /// Returns the stable diagnostic code for this error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::InvalidHandle { .. } => DiagnosticCode::CORE_INVALID_HANDLE,
            Self::NotFound { .. } => DiagnosticCode::CORE_NOT_FOUND,
            Self::NotImplemented { .. } => DiagnosticCode::CORE_NOT_IMPLEMENTED,
        }
    }

    /// Converts this error into a structured diagnostic with the provided source.
    #[must_use]
    pub fn to_diagnostic(
        &self,
        source: DiagnosticSource,
        blocking: Option<DiagnosticBlockingScope>,
    ) -> Diagnostic {
        let mut diagnostic = Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            source,
            self.to_string(),
        );
        diagnostic.blocking = blocking;
        diagnostic
    }
}

impl From<KinetikError> for Diagnostic {
    fn from(error: KinetikError) -> Self {
        error.to_diagnostic(DiagnosticSource::CORE, None)
    }
}

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(NonZeroU64);

        impl $name {
            /// Creates a new typed ID from a non-zero raw value.
            ///
            /// Kinetik reserves zero as invalid for every typed ID kind,
            /// including runtime IDs and stable GUID surrogates.
            ///
            /// # Panics
            ///
            /// Panics when `raw` is zero.
            #[must_use]
            pub const fn new(raw: u64) -> Self {
                match NonZeroU64::new(raw) {
                    Some(raw) => Self(raw),
                    None => panic!(concat!(stringify!($name), " raw value must be non-zero")),
                }
            }

            /// Returns the raw numeric value for serialization/debugging.
            #[must_use]
            pub const fn raw(self) -> u64 {
                self.0.get()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.raw())
            }
        }
    };
}

typed_id!(/// Runtime instance ID.
InstanceId);
typed_id!(/// Stable serialized instance GUID surrogate until UUID support lands.
InstanceGuid);
typed_id!(/// Runtime resource ID.
ResourceId);
typed_id!(/// Runtime signal ID.
SignalId);
typed_id!(/// Runtime script ID.
ScriptId);
typed_id!(/// Runtime bundle ID.
BundleId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_codes_are_stable_strings() {
        assert_eq!(
            DiagnosticCode::CORE_INVALID_HANDLE.as_str(),
            "KT_CORE_INVALID_HANDLE"
        );
        assert_eq!(DiagnosticCode::CORE_NOT_FOUND.as_str(), "KT_CORE_NOT_FOUND");
        assert_eq!(
            DiagnosticCode::CORE_NOT_IMPLEMENTED.as_str(),
            "KT_CORE_NOT_IMPLEMENTED"
        );
    }

    #[test]
    fn diagnostic_shape_carries_core_fields() {
        let location = DiagnosticLocation {
            instance_guid: Some(InstanceGuid::new(42)),
            scene_path: Some("/Game/Lighting/Sun".to_owned()),
            asset_path: None,
            script_path: Some("scripts/sun.luau".to_owned()),
            source_range: Some(SourceRange::new(1, 2, 3, 4)),
            property_path: Some("Intensity".to_owned()),
        };

        let diagnostic = Diagnostic::new(
            DiagnosticCode::new("KT_TEST_EXAMPLE"),
            DiagnosticSeverity::Warning,
            DiagnosticSource::new("Test"),
            "Example diagnostic",
        )
        .with_blocking_scope(DiagnosticBlockingScope::Test)
        .with_location(location)
        .with_suggested_fix("Update the test fixture")
        .allow_agent_repair();

        assert_eq!(diagnostic.code.as_str(), "KT_TEST_EXAMPLE");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostic.source.as_str(), "Test");
        assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Test));
        assert_eq!(diagnostic.location.instance_guid.unwrap().raw(), 42);
        assert_eq!(
            diagnostic.location.scene_path.as_deref(),
            Some("/Game/Lighting/Sun")
        );
        assert_eq!(
            diagnostic.location.script_path.as_deref(),
            Some("scripts/sun.luau")
        );
        assert_eq!(
            diagnostic.location.property_path.as_deref(),
            Some("Intensity")
        );
        assert_eq!(
            diagnostic.suggested_fix.as_deref(),
            Some("Update the test fixture")
        );
        assert_eq!(diagnostic.agent_repair, AgentRepair::Allowed);
    }

    #[test]
    fn kinetik_errors_map_to_diagnostics() {
        let error = KinetikError::InvalidHandle {
            kind: "InstanceId",
            id: 99,
        };

        let diagnostic = error.to_diagnostic(
            DiagnosticSource::new("Scene"),
            Some(DiagnosticBlockingScope::Play),
        );

        assert_eq!(diagnostic.code, DiagnosticCode::CORE_INVALID_HANDLE);
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.source.as_str(), "Scene");
        assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Play));
        assert_eq!(diagnostic.message, "invalid InstanceId handle: 99");
        assert_eq!(diagnostic.agent_repair, AgentRepair::NotAllowed);
    }

    #[test]
    fn kinetik_error_from_conversion_uses_core_source() {
        let diagnostic = Diagnostic::from(KinetikError::NotImplemented { feature: "Bundles" });

        assert_eq!(diagnostic.code, DiagnosticCode::CORE_NOT_IMPLEMENTED);
        assert_eq!(diagnostic.source, DiagnosticSource::CORE);
        assert_eq!(diagnostic.message, "feature not implemented: Bundles");
    }

    #[test]
    fn typed_ids_do_not_share_types() {
        let instance = InstanceId::new(7);
        let resource = ResourceId::new(7);
        assert_eq!(instance.raw(), resource.raw());
        assert_ne!(format!("{instance:?}"), format!("{resource:?}"));
    }

    #[test]
    fn typed_id_display_includes_kind_and_raw_value() {
        assert_eq!(InstanceId::new(1).to_string(), "InstanceId(1)");
        assert_eq!(InstanceGuid::new(2).to_string(), "InstanceGuid(2)");
        assert_eq!(ResourceId::new(3).to_string(), "ResourceId(3)");
        assert_eq!(SignalId::new(4).to_string(), "SignalId(4)");
        assert_eq!(ScriptId::new(5).to_string(), "ScriptId(5)");
        assert_eq!(BundleId::new(6).to_string(), "BundleId(6)");
    }

    #[test]
    fn typed_ids_reject_zero_raw_values() {
        assert!(std::panic::catch_unwind(|| InstanceId::new(0)).is_err());
        assert!(std::panic::catch_unwind(|| InstanceGuid::new(0)).is_err());
        assert!(std::panic::catch_unwind(|| ResourceId::new(0)).is_err());
        assert!(std::panic::catch_unwind(|| SignalId::new(0)).is_err());
        assert!(std::panic::catch_unwind(|| ScriptId::new(0)).is_err());
        assert!(std::panic::catch_unwind(|| BundleId::new(0)).is_err());
    }
}
