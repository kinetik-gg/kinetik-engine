use crate::{validate_command_kind, CommandError, CommandModelResult, CommandTargetMode};

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
