//! Validated command result contracts for Kinetik.
//!
//! Commands are the shared mutation surface for editor UI, MCP, dirty-state
//! tracking, undo/redo, validation, diagnostics, and tests. This crate starts
//! that surface with dependency-light result types; concrete mutation families
//! are intentionally left to focused follow-up issues.

use core::fmt;

use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};

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

/// Deterministic result returned by command validation or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    command_kind: String,
    target_mode: Option<CommandTargetMode>,
    status: CommandStatus,
    diagnostics: Vec<Diagnostic>,
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

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-command"
}

#[cfg(test)]
mod tests;
