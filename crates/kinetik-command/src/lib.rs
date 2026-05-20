//! Validated command result contracts for Kinetik.
//!
//! Commands are the shared mutation surface for editor UI, MCP, dirty-state
//! tracking, undo/redo, validation, diagnostics, and tests. This crate starts
//! that surface with dependency-light result types; concrete mutation families
//! are intentionally left to focused follow-up issues.

use core::{fmt, num::NonZeroU64};

use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
    InstanceGuid,
};
use kinetik_reflect::PropertyValue;

/// Result type for command model operations.
pub type CommandModelResult<T> = Result<T, CommandError>;

/// Stable command target mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandTargetMode {
    /// Command targets editable source/project state.
    Edit,
    /// Command targets sandboxed runtime/play state.
    Play,
}

impl fmt::Display for CommandTargetMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit => f.write_str("edit"),
            Self::Play => f.write_str("play"),
        }
    }
}

/// Command execution status.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandStatus {
    /// Command validated and completed.
    Succeeded,
    /// Command was rejected before mutation.
    Failed,
}

/// Errors returned while building or validating command model values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    /// Command name was empty.
    EmptyCommandKind,
    /// Dirty-state summary text was empty.
    EmptyDirtySummary,
    /// Command needed a target mode but none was provided.
    AmbiguousTargetMode {
        /// Command kind that needed a mode.
        command_kind: String,
    },
    /// Command was invoked against the wrong target mode.
    WrongTargetMode {
        /// Command kind that rejected the mode.
        command_kind: String,
        /// Required command target mode.
        expected: CommandTargetMode,
        /// Actual command target mode.
        actual: CommandTargetMode,
    },
}

impl CommandError {
    /// Stable diagnostic code for empty command kinds.
    pub const EMPTY_COMMAND_KIND_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_COMMAND_EMPTY_KIND");

    /// Stable diagnostic code for empty dirty-state summaries.
    pub const EMPTY_DIRTY_SUMMARY_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_COMMAND_EMPTY_DIRTY_SUMMARY");

    /// Stable diagnostic code for ambiguous edit/play command target mode.
    pub const AMBIGUOUS_TARGET_MODE_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_COMMAND_AMBIGUOUS_TARGET_MODE");

    /// Stable diagnostic code for commands sent to the wrong target mode.
    pub const WRONG_TARGET_MODE_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_COMMAND_WRONG_TARGET_MODE");

    /// Diagnostic source for command-owned validation.
    pub const COMMAND_SOURCE: DiagnosticSource = DiagnosticSource::new("Command");

    /// Returns the stable diagnostic code for this command error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::EmptyCommandKind => Self::EMPTY_COMMAND_KIND_CODE,
            Self::EmptyDirtySummary => Self::EMPTY_DIRTY_SUMMARY_CODE,
            Self::AmbiguousTargetMode { .. } => Self::AMBIGUOUS_TARGET_MODE_CODE,
            Self::WrongTargetMode { .. } => Self::WRONG_TARGET_MODE_CODE,
        }
    }

    /// Converts this error into a structured diagnostic.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            Self::COMMAND_SOURCE,
            self.to_string(),
        )
        .with_blocking_scope(DiagnosticBlockingScope::Edit)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCommandKind => f.write_str("command kind must not be empty"),
            Self::EmptyDirtySummary => f.write_str("dirty-state summary must not be empty"),
            Self::AmbiguousTargetMode { command_kind } => write!(
                f,
                "command target mode is ambiguous for command: {command_kind}"
            ),
            Self::WrongTargetMode {
                command_kind,
                expected,
                actual,
            } => write!(
                f,
                "command {command_kind} targets {actual} mode but requires {expected} mode"
            ),
        }
    }
}

impl std::error::Error for CommandError {}

/// User-facing undo group identity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UndoGroupId(NonZeroU64);

impl UndoGroupId {
    /// Creates an undo group ID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        let Some(raw) = NonZeroU64::new(raw) else {
            panic!("UndoGroupId raw value must be non-zero");
        };
        Self(raw)
    }

    /// Returns the raw non-zero ID value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for UndoGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UndoGroupId({})", self.raw())
    }
}

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
    /// Returns [`CommandError`] when required text fields are empty.
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

/// Deterministic result returned by command validation or execution.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    command_kind: String,
    target_mode: Option<CommandTargetMode>,
    status: CommandStatus,
    diagnostics: Vec<Diagnostic>,
    changes: Vec<CommandChangeRecord>,
}

impl CommandResult {
    /// Creates a successful command result.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::EmptyCommandKind`] when `command_kind` is empty.
    pub fn succeeded(
        command_kind: impl Into<String>,
        target_mode: CommandTargetMode,
    ) -> CommandModelResult<Self> {
        Ok(Self {
            command_kind: validate_command_kind(command_kind.into())?,
            target_mode: Some(target_mode),
            status: CommandStatus::Succeeded,
            diagnostics: Vec::new(),
            changes: Vec::new(),
        })
    }

    /// Creates a successful command result with semantic change records.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::EmptyCommandKind`] when `command_kind` is empty.
    pub fn succeeded_with_changes<I>(
        command_kind: impl Into<String>,
        target_mode: CommandTargetMode,
        changes: I,
    ) -> CommandModelResult<Self>
    where
        I: IntoIterator<Item = CommandChangeRecord>,
    {
        Ok(Self {
            command_kind: validate_command_kind(command_kind.into())?,
            target_mode: Some(target_mode),
            status: CommandStatus::Succeeded,
            diagnostics: Vec::new(),
            changes: changes.into_iter().collect(),
        })
    }

    /// Creates a failed command result from deterministic diagnostics.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::EmptyCommandKind`] when `command_kind` is empty.
    pub fn failed<I>(
        command_kind: impl Into<String>,
        target_mode: Option<CommandTargetMode>,
        diagnostics: I,
    ) -> CommandModelResult<Self>
    where
        I: IntoIterator<Item = Diagnostic>,
    {
        Ok(Self {
            command_kind: validate_command_kind(command_kind.into())?,
            target_mode,
            status: CommandStatus::Failed,
            diagnostics: diagnostics.into_iter().collect(),
            changes: Vec::new(),
        })
    }

    /// Creates a failed command result from a command error.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::EmptyCommandKind`] when `command_kind` is empty.
    pub fn rejected(
        command_kind: impl Into<String>,
        target_mode: Option<CommandTargetMode>,
        error: &CommandError,
    ) -> CommandModelResult<Self> {
        Self::failed(command_kind, target_mode, [error.to_diagnostic()])
    }

    /// Returns the stable command kind.
    #[must_use]
    pub fn command_kind(&self) -> &str {
        &self.command_kind
    }

    /// Returns the command target mode when known.
    #[must_use]
    pub const fn target_mode(&self) -> Option<CommandTargetMode> {
        self.target_mode
    }

    /// Returns the command status.
    #[must_use]
    pub const fn status(&self) -> CommandStatus {
        self.status
    }

    /// Returns whether this result succeeded.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self.status, CommandStatus::Succeeded)
    }

    /// Returns whether this result failed before mutation.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self.status, CommandStatus::Failed)
    }

    /// Returns diagnostics in deterministic command validation order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns semantic change records in deterministic command execution order.
    #[must_use]
    pub fn changes(&self) -> &[CommandChangeRecord] {
        &self.changes
    }
}

/// Requires an explicit command target mode.
///
/// # Errors
///
/// Returns [`CommandError::AmbiguousTargetMode`] when `target_mode` is `None`.
pub fn require_target_mode(
    command_kind: impl Into<String>,
    target_mode: Option<CommandTargetMode>,
) -> CommandModelResult<CommandTargetMode> {
    let command_kind = validate_command_kind(command_kind.into())?;
    target_mode.ok_or(CommandError::AmbiguousTargetMode { command_kind })
}

/// Requires a specific target mode for a command.
///
/// # Errors
///
/// Returns [`CommandError`] when the command kind is empty, the mode is
/// ambiguous, or the provided mode does not match `expected`.
pub fn require_specific_target_mode(
    command_kind: impl Into<String>,
    target_mode: Option<CommandTargetMode>,
    expected: CommandTargetMode,
) -> CommandModelResult<CommandTargetMode> {
    let command_kind = validate_command_kind(command_kind.into())?;
    let actual = target_mode.ok_or_else(|| CommandError::AmbiguousTargetMode {
        command_kind: command_kind.clone(),
    })?;
    if actual == expected {
        Ok(actual)
    } else {
        Err(CommandError::WrongTargetMode {
            command_kind,
            expected,
            actual,
        })
    }
}

fn validate_command_kind(command_kind: String) -> CommandModelResult<String> {
    if command_kind.trim().is_empty() {
        Err(CommandError::EmptyCommandKind)
    } else {
        Ok(command_kind)
    }
}

fn validate_dirty_summary(dirty_summary: String) -> CommandModelResult<String> {
    if dirty_summary.trim().is_empty() {
        Err(CommandError::EmptyDirtySummary)
    } else {
        Ok(dirty_summary)
    }
}

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-command"
}

#[cfg(test)]
mod tests;
