//! Validated command result contracts for Kinetik.
//!
//! Commands are the shared mutation surface for editor UI, MCP, dirty-state
//! tracking, undo/redo, validation, diagnostics, and tests. This crate starts
//! that surface with dependency-light model types; concrete mutation families
//! are intentionally left to focused follow-up issues.

mod asset;
mod change;
mod dirty;
mod error;
mod history;
mod mode;
mod result;
mod scene;
mod script;
mod target_mode;

pub use asset::{
    request_asset_reimport, update_asset_path, AssetPathCommandResult, AssetReimportCommandResult,
    REIMPORT_ASSET_COMMAND, UPDATE_ASSET_PATH_COMMAND,
};
pub use change::{ChangeTarget, CommandChangeRecord, PropertyValueChange};
pub use dirty::{DirtyChangeExplanation, DirtyDocumentExplanation, DirtyStateExplanation};
pub use error::{CommandError, CommandModelResult};
pub use history::{CommandHistory, UndoGroupId, UndoRedoRecord};
pub use mode::{CommandStatus, CommandTargetMode};
pub use result::CommandResult;
pub use scene::{
    create_scene_child_instance, delete_scene_instance, duplicate_scene_instance,
    rename_scene_instance, reparent_scene_instance, set_scene_instance_property,
    SceneCreateCommandResult, SceneDeleteCommandResult, SceneDuplicateCommandResult,
    SceneReparentCommandResult, SceneSetPropertyCommandResult, CREATE_INSTANCE_COMMAND,
    DELETE_INSTANCE_COMMAND, DUPLICATE_INSTANCE_COMMAND, RENAME_INSTANCE_COMMAND,
    REPARENT_INSTANCE_COMMAND, SET_PROPERTY_COMMAND,
};
pub use script::{
    attach_instance_script, detach_instance_script, ScriptAttachCommandResult,
    ScriptAttachmentDocument, ScriptDetachCommandResult, ATTACH_SCRIPT_COMMAND,
    DETACH_SCRIPT_COMMAND,
};
pub use target_mode::{require_specific_target_mode, require_target_mode};

pub(crate) use error::{validate_command_kind, validate_dirty_summary};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-command"
}

#[cfg(test)]
mod tests;
