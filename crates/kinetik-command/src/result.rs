use kinetik_core::Diagnostic;

use crate::{
    validate_command_kind, CommandChangeRecord, CommandError, CommandModelResult, CommandStatus,
    CommandTargetMode,
};

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
