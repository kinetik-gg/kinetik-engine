use core::fmt;

use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};

use crate::CommandTargetMode;

/// Result type for command model operations.
pub type CommandModelResult<T> = Result<T, CommandError>;

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
    /// Command validation failed before mutation.
    ValidationFailed {
        /// Command kind that failed validation.
        command_kind: String,
        /// Validation failure reason.
        reason: String,
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

    /// Stable diagnostic code for command validation failures.
    pub const VALIDATION_FAILED_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_COMMAND_VALIDATION_FAILED");

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
            Self::ValidationFailed { .. } => Self::VALIDATION_FAILED_CODE,
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
            Self::ValidationFailed {
                command_kind,
                reason,
            } => write!(f, "command {command_kind} failed validation: {reason}"),
        }
    }
}

impl std::error::Error for CommandError {}

pub(crate) fn validate_command_kind(command_kind: String) -> CommandModelResult<String> {
    if command_kind.trim().is_empty() {
        Err(CommandError::EmptyCommandKind)
    } else {
        Ok(command_kind)
    }
}

pub(crate) fn validate_dirty_summary(dirty_summary: String) -> CommandModelResult<String> {
    if dirty_summary.trim().is_empty() {
        Err(CommandError::EmptyDirtySummary)
    } else {
        Ok(dirty_summary)
    }
}
