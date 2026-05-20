use kinetik_command::{
    create_scene_child_instance, delete_scene_instance, duplicate_scene_instance,
    rename_scene_instance, reparent_scene_instance, require_specific_target_mode,
    set_scene_instance_property, CommandError, CommandHistory, CommandResult, CommandStatus,
    CommandTargetMode, DirtyStateExplanation, UndoRedoRecord, CREATE_INSTANCE_COMMAND,
    DELETE_INSTANCE_COMMAND, DUPLICATE_INSTANCE_COMMAND, RENAME_INSTANCE_COMMAND,
    REPARENT_INSTANCE_COMMAND, SET_PROPERTY_COMMAND,
};
use kinetik_reflect::PropertyValue;
use kinetik_scene::Scene;

use super::{
    diagnostics_list_response, dirty_state_response, DiagnosticSummary, DirtyStateResponse,
};

/// Editor-owned mutating MCP command names.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum McpMutatingCommand {
    /// Create a child instance.
    SceneCreateInstance,
    /// Delete an instance subtree.
    SceneDeleteInstance,
    /// Rename an instance.
    SceneRenameInstance,
    /// Reparent an instance.
    SceneReparentInstance,
    /// Duplicate an instance subtree.
    SceneDuplicateInstance,
    /// Set a reflected property.
    SceneSetProperty,
    /// Move the latest command history record to the redo stack.
    EditorUndo,
    /// Move the latest redo record back to the undo stack.
    EditorRedo,
}

impl McpMutatingCommand {
    /// Returns the stable MCP command name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SceneCreateInstance => "scene.create_instance",
            Self::SceneDeleteInstance => "scene.delete_instance",
            Self::SceneRenameInstance => "scene.rename_instance",
            Self::SceneReparentInstance => "scene.reparent_instance",
            Self::SceneDuplicateInstance => "scene.duplicate_instance",
            Self::SceneSetProperty => "scene.set_property",
            Self::EditorUndo => "editor.undo",
            Self::EditorRedo => "editor.redo",
        }
    }
}

/// Mutating scene request mapped to editor command APIs.
#[derive(Debug, Clone, PartialEq)]
pub enum McpSceneMutationRequest {
    /// Create a child instance.
    CreateInstance {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
        /// Parent instance ID.
        parent_id: kinetik_core::InstanceId,
        /// Class name to create.
        class_name: String,
        /// Instance name.
        name: String,
    },
    /// Delete an instance subtree.
    DeleteInstance {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
        /// Instance ID to delete.
        instance_id: kinetik_core::InstanceId,
    },
    /// Rename an instance.
    RenameInstance {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
        /// Instance ID to rename.
        instance_id: kinetik_core::InstanceId,
        /// New instance name.
        new_name: String,
    },
    /// Reparent an instance.
    ReparentInstance {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
        /// Instance ID to reparent.
        instance_id: kinetik_core::InstanceId,
        /// New parent instance ID.
        new_parent: kinetik_core::InstanceId,
    },
    /// Duplicate an instance subtree.
    DuplicateInstance {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
        /// Source instance ID.
        instance_id: kinetik_core::InstanceId,
        /// Parent for the duplicated root.
        new_parent: kinetik_core::InstanceId,
    },
    /// Set a reflected property.
    SetProperty {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
        /// Instance ID receiving the property write.
        instance_id: kinetik_core::InstanceId,
        /// Canonical reflected property path.
        property_path: String,
        /// New reflected value.
        value: PropertyValue,
    },
}

/// Mutating MCP command response.
#[derive(Debug, Clone, PartialEq)]
pub struct McpMutationResponse {
    /// Stable command kind.
    pub command_kind: String,
    /// Command target mode.
    pub target_mode: Option<CommandTargetMode>,
    /// Command status.
    pub status: CommandStatus,
    /// Diagnostics returned by validation or execution.
    pub diagnostics: Vec<DiagnosticSummary>,
    /// Dirty/change summaries produced by the command.
    pub change_summaries: Vec<String>,
    /// Undo group assigned after command history commit.
    pub undo_group: Option<u64>,
    /// Dirty state after command execution.
    pub dirty_state: DirtyStateResponse,
}

/// Undo/redo MCP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpUndoRedoResponse {
    /// Whether a history record moved.
    pub moved: bool,
    /// Undo group that moved, when any.
    pub undo_group: Option<u64>,
    /// User-facing command summary, when any.
    pub summary: Option<String>,
    /// Dirty state after stack movement.
    pub dirty_state: DirtyStateResponse,
}

/// Editor-owned mutating MCP scaffold session.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpMutationSession {
    history: CommandHistory,
}

impl McpMutationSession {
    /// Creates an empty mutating MCP session.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            history: CommandHistory::new(),
        }
    }

    /// Returns command history.
    #[must_use]
    pub const fn history(&self) -> &CommandHistory {
        &self.history
    }

    /// Executes a scene mutation through the shared command path.
    pub fn execute_scene_mutation(
        &mut self,
        scene: &mut Scene,
        request: McpSceneMutationRequest,
        document_path: impl Into<String>,
    ) -> McpMutationResponse {
        let document_path = document_path.into();
        let command_kind = request.command_kind();
        let target_mode = request.target_mode();
        let result =
            require_specific_target_mode(command_kind, target_mode, CommandTargetMode::Edit)
                .and_then(|_| execute_scene_command(scene, request, &document_path));
        self.response_from_result(command_kind, target_mode, result)
    }

    /// Moves the latest undo record to the redo stack.
    pub fn undo(&mut self) -> McpUndoRedoResponse {
        let record = self.history.pop_undo();
        self.undo_redo_response(record)
    }

    /// Moves the latest redo record back to the undo stack.
    pub fn redo(&mut self) -> McpUndoRedoResponse {
        let record = self.history.pop_redo();
        self.undo_redo_response(record)
    }

    fn response_from_result(
        &mut self,
        command_kind: &str,
        target_mode: Option<CommandTargetMode>,
        result: Result<CommandResult, CommandError>,
    ) -> McpMutationResponse {
        let command = result.unwrap_or_else(|error| {
            CommandResult::rejected(command_kind, target_mode, &error)
                .expect("MCP command kind constants should be valid")
        });
        let history_record = self
            .history
            .commit_result(command.command_kind(), &command)
            .expect("MCP command summaries should be valid");
        let dirty_state = dirty_state_response(&DirtyStateExplanation::from_history(&self.history));

        McpMutationResponse {
            command_kind: command.command_kind().to_owned(),
            target_mode: command.target_mode(),
            status: command.status(),
            diagnostics: diagnostics_list_response(command.diagnostics()),
            change_summaries: command
                .changes()
                .iter()
                .map(|change| change.dirty_summary().to_owned())
                .collect(),
            undo_group: history_record.map(|record| record.group_id().raw()),
            dirty_state,
        }
    }

    fn undo_redo_response(&self, record: Option<UndoRedoRecord>) -> McpUndoRedoResponse {
        let dirty_state = dirty_state_response(&DirtyStateExplanation::from_history(&self.history));
        McpUndoRedoResponse {
            moved: record.is_some(),
            undo_group: record.as_ref().map(|record| record.group_id().raw()),
            summary: record.map(|record| record.summary().to_owned()),
            dirty_state,
        }
    }
}

impl McpSceneMutationRequest {
    fn command_kind(&self) -> &'static str {
        match self {
            Self::CreateInstance { .. } => CREATE_INSTANCE_COMMAND,
            Self::DeleteInstance { .. } => DELETE_INSTANCE_COMMAND,
            Self::RenameInstance { .. } => RENAME_INSTANCE_COMMAND,
            Self::ReparentInstance { .. } => REPARENT_INSTANCE_COMMAND,
            Self::DuplicateInstance { .. } => DUPLICATE_INSTANCE_COMMAND,
            Self::SetProperty { .. } => SET_PROPERTY_COMMAND,
        }
    }

    const fn target_mode(&self) -> Option<CommandTargetMode> {
        match self {
            Self::CreateInstance { target_mode, .. }
            | Self::DeleteInstance { target_mode, .. }
            | Self::RenameInstance { target_mode, .. }
            | Self::ReparentInstance { target_mode, .. }
            | Self::DuplicateInstance { target_mode, .. }
            | Self::SetProperty { target_mode, .. } => *target_mode,
        }
    }
}

fn execute_scene_command(
    scene: &mut Scene,
    request: McpSceneMutationRequest,
    document_path: &str,
) -> Result<CommandResult, CommandError> {
    match request {
        McpSceneMutationRequest::CreateInstance {
            parent_id,
            class_name,
            name,
            ..
        } => create_scene_child_instance(scene, parent_id, class_name, name, document_path)
            .map(|result| result.command),
        McpSceneMutationRequest::DeleteInstance { instance_id, .. } => {
            delete_scene_instance(scene, instance_id, document_path).map(|result| result.command)
        }
        McpSceneMutationRequest::RenameInstance {
            instance_id,
            new_name,
            ..
        } => rename_scene_instance(scene, instance_id, new_name, document_path),
        McpSceneMutationRequest::ReparentInstance {
            instance_id,
            new_parent,
            ..
        } => reparent_scene_instance(scene, instance_id, new_parent, document_path)
            .map(|result| result.command),
        McpSceneMutationRequest::DuplicateInstance {
            instance_id,
            new_parent,
            ..
        } => duplicate_scene_instance(scene, instance_id, new_parent, document_path)
            .map(|result| result.command),
        McpSceneMutationRequest::SetProperty {
            instance_id,
            property_path,
            value,
            ..
        } => set_scene_instance_property(scene, instance_id, property_path, value, document_path)
            .map(|result| result.command),
    }
}
