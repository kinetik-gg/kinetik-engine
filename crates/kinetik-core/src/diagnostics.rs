use core::fmt;

use crate::InstanceGuid;

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
