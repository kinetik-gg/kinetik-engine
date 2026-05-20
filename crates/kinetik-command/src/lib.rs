//! Validated command result contracts for Kinetik.
//!
//! Commands are the shared mutation surface for editor UI, MCP, dirty-state
//! tracking, undo/redo, validation, diagnostics, and tests. This crate starts
//! that surface with dependency-light model types; concrete mutation families
//! are intentionally left to focused follow-up issues.

mod change;
mod error;
mod history;
mod mode;
mod result;
mod target_mode;

pub use change::{ChangeTarget, CommandChangeRecord, PropertyValueChange};
pub use error::{CommandError, CommandModelResult};
pub use history::{CommandHistory, UndoGroupId, UndoRedoRecord};
pub use mode::{CommandStatus, CommandTargetMode};
pub use result::CommandResult;
pub use target_mode::{require_specific_target_mode, require_target_mode};

pub(crate) use error::{validate_command_kind, validate_dirty_summary};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-command"
}

#[cfg(test)]
mod tests;
