mod mutating;
mod play;

pub use mutating::{
    McpMutatingCommand, McpMutationResponse, McpMutationSession, McpSceneMutationRequest,
    McpUndoRedoResponse,
};
pub use play::{McpPlayCommand, McpPlayCommandName, McpPlayCommandResponse, McpPlayStateResponse};

use kinetik_command::{
    require_specific_target_mode, CommandError, CommandResult, CommandTargetMode,
    DirtyStateExplanation, UndoRedoRecord,
};
use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticSeverity, InstanceGuid, InstanceId,
};
use kinetik_project::{ProjectDocumentRefs, ProjectSettingsDocument};
use kinetik_resource::AssetManifest;
use kinetik_scene::Scene;

use crate::{EditorDocumentSelection, EditorPanel, EditorSession};
use crate::{ViewportPickRequest, ViewportPickResponse, ViewportSnapshot};
use mutating::execute_scene_command;

/// Editor-owned read-only MCP command names.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum McpReadOnlyCommand {
    /// Inspect project identity and active source documents.
    ProjectStatus,
    /// List scene instances in deterministic hierarchy order.
    SceneListInstances,
    /// List resource manifest entries in deterministic manifest order.
    ResourceList,
    /// List current diagnostics.
    DiagnosticsList,
    /// Explain current edit dirty state.
    DirtyState,
}

impl McpReadOnlyCommand {
    /// Returns the stable MCP command name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProjectStatus => "project.status",
            Self::SceneListInstances => "scene.list_instances",
            Self::ResourceList => "resource.list",
            Self::DiagnosticsList => "diagnostics.list",
            Self::DirtyState => "editor.dirty_state",
        }
    }
}

/// Read-only project status response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectStatusResponse {
    /// Project display name.
    pub project_name: String,
    /// Engine compatibility string.
    pub engine_compatibility: String,
    /// Active scene source document path.
    pub active_scene: String,
    /// Asset manifest source document path.
    pub assets_manifest: String,
    /// Instance manifest source document path.
    pub instances_manifest: String,
}

/// Read-only scene instance summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneInstanceSummary {
    /// Runtime/editor session instance ID raw value.
    pub instance_id: u64,
    /// Stable instance GUID raw value.
    pub guid: u64,
    /// Registered class name.
    pub class_name: String,
    /// Display instance name.
    pub name: String,
    /// Human-readable scene path.
    pub scene_path: String,
    /// Parent instance ID raw value.
    pub parent_id: Option<u64>,
    /// Child instance IDs in scene order.
    pub child_ids: Vec<u64>,
}

/// Read-only resource manifest entry summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManifestEntrySummary {
    /// Stable asset GUID raw value.
    pub guid: u64,
    /// Project resource path.
    pub path: String,
    /// Importer identifier.
    pub importer_id: String,
    /// Importer version.
    pub importer_version: String,
    /// Import settings hash.
    pub settings_hash: String,
}

/// Read-only resource manifest response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManifestResponse {
    /// Manifest entries in deterministic path order.
    pub entries: Vec<ResourceManifestEntrySummary>,
}

/// Read-only diagnostic summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSummary {
    /// Stable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// System that produced the diagnostic.
    pub source: String,
    /// Human-readable message.
    pub message: String,
    /// Blocked workflow, when known.
    pub blocking: Option<DiagnosticBlockingScope>,
    /// Related instance GUID raw value, when known.
    pub instance_guid: Option<u64>,
    /// Related scene path, when known.
    pub scene_path: Option<String>,
    /// Related asset path, when known.
    pub asset_path: Option<String>,
    /// Related script path, when known.
    pub script_path: Option<String>,
    /// Related property path, when known.
    pub property_path: Option<String>,
}

/// Read-only dirty-state response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyStateResponse {
    /// Whether any edit document is dirty.
    pub is_dirty: bool,
    /// Dirty document paths in first-change order.
    pub documents: Vec<String>,
    /// Human-readable dirty summaries in command history order.
    pub summaries: Vec<String>,
}

/// Read-only editor selection response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpSelectionResponse {
    /// No document or scene instance is selected.
    None,
    /// A scene instance is selected in edit state.
    SceneInstance {
        /// Runtime/editor session instance ID raw value.
        instance_id: u64,
        /// Stable serialized instance GUID raw value.
        guid: u64,
        /// Human-readable scene path.
        scene_path: String,
    },
    /// A project document is selected.
    ProjectDocument {
        /// Workspace-relative project document path.
        path: String,
    },
}

/// Editor session snapshot exposed to agent-facing MCP helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct McpEditorSnapshot {
    /// Active project status, when a project is open.
    pub project: Option<ProjectStatusResponse>,
    /// Active scene hierarchy, empty when no scene is open or projection fails.
    pub scene: Vec<SceneInstanceSummary>,
    /// Active resource manifest entries.
    pub resources: ResourceManifestResponse,
    /// Current selection.
    pub selection: McpSelectionResponse,
    /// Focused editor panel.
    pub focus: Option<EditorPanel>,
    /// Current diagnostics.
    pub diagnostics: Vec<DiagnosticSummary>,
    /// Current dirty-state explanation.
    pub dirty_state: DirtyStateResponse,
    /// Current play-mode runtime state.
    pub play_state: McpPlayStateResponse,
    /// Renderer-independent viewport interaction state.
    pub viewport: ViewportSnapshot,
}

/// MCP/headless viewport focus response.
#[derive(Debug, Clone, PartialEq)]
pub struct McpViewportFocusResponse {
    /// Viewport snapshot after the focus command.
    pub viewport: ViewportSnapshot,
    /// Validation diagnostics, when focus failed.
    pub diagnostics: Vec<DiagnosticSummary>,
}

/// MCP/headless viewport picking response.
#[derive(Debug, Clone, PartialEq)]
pub struct McpViewportPickResponse {
    /// Placeholder picking response.
    pub pick: ViewportPickResponse,
    /// Viewport snapshot at pick time.
    pub viewport: ViewportSnapshot,
}

/// Selection and focus requests that MCP can apply to the editor session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpSelectionRequest {
    /// Select a scene instance by runtime/editor session ID.
    SelectSceneInstance {
        /// Instance ID to select.
        instance_id: InstanceId,
    },
    /// Select a project document.
    SelectProjectDocument {
        /// Workspace-relative project document path.
        path: String,
    },
    /// Focus an editor panel without changing selection.
    FocusPanel {
        /// Panel to focus.
        panel: EditorPanel,
    },
    /// Clear current selection and focus.
    Clear,
}

/// Selection/focus command response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpSelectionCommandResponse {
    /// Selection after applying the request.
    pub selection: McpSelectionResponse,
    /// Focus after applying the request.
    pub focus: Option<EditorPanel>,
    /// Validation diagnostics, when the request failed.
    pub diagnostics: Vec<DiagnosticSummary>,
}

/// Builds a read-only project status response.
#[must_use]
pub fn project_status_response(
    settings: &ProjectSettingsDocument,
    documents: &ProjectDocumentRefs,
) -> ProjectStatusResponse {
    ProjectStatusResponse {
        project_name: settings.identity().name().to_owned(),
        engine_compatibility: settings.identity().engine_compatibility().to_owned(),
        active_scene: documents.active_scene().to_owned(),
        assets_manifest: documents.assets_manifest().to_owned(),
        instances_manifest: documents.instances_manifest().to_owned(),
    }
}

/// Builds a deterministic read-only scene hierarchy response.
///
/// # Errors
///
/// Returns scene errors when the scene has invalid parent/child references.
pub fn scene_hierarchy_response(
    scene: &Scene,
) -> kinetik_scene::SceneResult<Vec<SceneInstanceSummary>> {
    let mut summaries = Vec::new();
    if let Some(root_id) = scene.root_id() {
        collect_instance_summary(scene, root_id, &mut summaries)?;
    }
    Ok(summaries)
}

/// Builds a deterministic read-only resource manifest response.
#[must_use]
pub fn resource_manifest_response(manifest: &AssetManifest) -> ResourceManifestResponse {
    ResourceManifestResponse {
        entries: manifest
            .entries()
            .iter()
            .map(|entry| ResourceManifestEntrySummary {
                guid: entry.guid().raw(),
                path: entry.path().as_str().to_owned(),
                importer_id: entry.importer_id().to_owned(),
                importer_version: entry.importer_version().to_owned(),
                settings_hash: entry.settings_hash().as_str().to_owned(),
            })
            .collect(),
    }
}

/// Builds a read-only diagnostics list response.
#[must_use]
pub fn diagnostics_list_response(diagnostics: &[Diagnostic]) -> Vec<DiagnosticSummary> {
    diagnostics
        .iter()
        .map(|diagnostic| DiagnosticSummary {
            code: diagnostic.code.as_str().to_owned(),
            severity: diagnostic.severity,
            source: diagnostic.source.as_str().to_owned(),
            message: diagnostic.message.clone(),
            blocking: diagnostic.blocking,
            instance_guid: diagnostic.location.instance_guid.map(InstanceGuid::raw),
            scene_path: diagnostic.location.scene_path.clone(),
            asset_path: diagnostic.location.asset_path.clone(),
            script_path: diagnostic.location.script_path.clone(),
            property_path: diagnostic.location.property_path.clone(),
        })
        .collect()
}

/// Builds a read-only dirty-state response.
#[must_use]
pub fn dirty_state_response(explanation: &DirtyStateExplanation) -> DirtyStateResponse {
    let documents = explanation
        .documents()
        .iter()
        .map(|document| document.document_path().to_owned())
        .collect::<Vec<_>>();
    let summaries = explanation
        .changes()
        .iter()
        .map(|change| change.change_summary().to_owned())
        .collect::<Vec<_>>();

    DirtyStateResponse {
        is_dirty: !documents.is_empty(),
        documents,
        summaries,
    }
}

/// Builds a read-only selection response.
#[must_use]
pub fn selection_response(selection: &EditorDocumentSelection) -> McpSelectionResponse {
    match selection {
        EditorDocumentSelection::None => McpSelectionResponse::None,
        EditorDocumentSelection::SceneInstance {
            id,
            guid,
            scene_path,
        } => McpSelectionResponse::SceneInstance {
            instance_id: id.raw(),
            guid: guid.raw(),
            scene_path: scene_path.clone(),
        },
        EditorDocumentSelection::ProjectDocument { path } => {
            McpSelectionResponse::ProjectDocument { path: path.clone() }
        }
    }
}

impl EditorSession {
    /// Returns a complete MCP snapshot of the current editor session state.
    #[must_use]
    pub fn mcp_snapshot(&self) -> McpEditorSnapshot {
        let project = self
            .project()
            .map(|project| project_status_response(project.settings(), project.documents()));
        let scene = self
            .active_scene()
            .and_then(|scene| scene_hierarchy_response(scene).ok())
            .unwrap_or_default();
        let diagnostics = self
            .diagnostics_panel()
            .items()
            .iter()
            .map(|item| DiagnosticSummary {
                code: item.code.clone(),
                severity: item.severity,
                source: item.source.clone(),
                message: item.message.clone(),
                blocking: item.blocking,
                instance_guid: None,
                scene_path: None,
                asset_path: None,
                script_path: None,
                property_path: None,
            })
            .collect();

        McpEditorSnapshot {
            project,
            scene,
            resources: resource_manifest_response(self.asset_manifest()),
            selection: selection_response(self.selection().document()),
            focus: self.selection().focus().panel(),
            diagnostics,
            dirty_state: dirty_state_response(&self.dirty_state()),
            play_state: self.mcp_play_state(),
            viewport: self.viewport_snapshot(),
        }
    }

    /// Focuses the viewport on the current scene selection.
    pub fn mcp_viewport_focus_selected(&mut self) -> McpViewportFocusResponse {
        let diagnostics = self
            .viewport_focus_selected()
            .err()
            .map(|error| {
                vec![
                    mcp_validation_error("viewport.focus_selected", error.to_string())
                        .to_diagnostic(),
                ]
            })
            .unwrap_or_default();
        McpViewportFocusResponse {
            viewport: self.viewport_snapshot(),
            diagnostics: diagnostics_list_response(&diagnostics),
        }
    }

    /// Applies the current viewport picking contract.
    #[must_use]
    pub fn mcp_viewport_pick(&self, request: ViewportPickRequest) -> McpViewportPickResponse {
        McpViewportPickResponse {
            pick: self.viewport_pick(request),
            viewport: self.viewport_snapshot(),
        }
    }

    /// Applies a selection/focus request through the editor session authority.
    pub fn mcp_apply_selection(
        &mut self,
        request: McpSelectionRequest,
    ) -> McpSelectionCommandResponse {
        let result = match request {
            McpSelectionRequest::SelectSceneInstance { instance_id } => self
                .active_scene()
                .cloned()
                .ok_or_else(|| mcp_validation_error("editor.select", "no active scene is open"))
                .and_then(|scene| {
                    self.selection_mut()
                        .select_scene_instance(&scene, instance_id)
                        .map_err(|error| mcp_validation_error("editor.select", error.to_string()))
                }),
            McpSelectionRequest::SelectProjectDocument { path } => {
                self.selection_mut().select_project_document(path);
                Ok(())
            }
            McpSelectionRequest::FocusPanel { panel } => {
                self.selection_mut().focus_panel(panel);
                Ok(())
            }
            McpSelectionRequest::Clear => {
                self.selection_mut().clear();
                Ok(())
            }
        };
        let diagnostics = result
            .err()
            .map(|error| vec![error.to_diagnostic()])
            .unwrap_or_default();
        McpSelectionCommandResponse {
            selection: selection_response(self.selection().document()),
            focus: self.selection().focus().panel(),
            diagnostics: diagnostics_list_response(&diagnostics),
        }
    }

    /// Executes a scene mutation through the session-owned command history.
    pub fn mcp_execute_scene_mutation(
        &mut self,
        request: McpSceneMutationRequest,
    ) -> McpMutationResponse {
        let command_kind = request.command_kind();
        let target_mode = request.target_mode();
        let result =
            require_specific_target_mode(command_kind, target_mode, CommandTargetMode::Edit)
                .and_then(|_| {
                    let document_path = self
                        .project()
                        .ok_or_else(|| mcp_validation_error(command_kind, "no project is open"))?
                        .documents()
                        .active_scene()
                        .to_owned();
                    let scene = self.active_scene_mut().ok_or_else(|| {
                        mcp_validation_error(command_kind, "no active scene is open")
                    })?;
                    execute_scene_command(scene, request, &document_path)
                });
        self.mcp_response_from_result(command_kind, target_mode, result)
    }

    /// Moves the latest editor session undo record to the redo stack.
    pub fn mcp_undo(&mut self) -> McpUndoRedoResponse {
        let record = self.command_history_mut().pop_undo();
        self.mcp_undo_redo_response(record)
    }

    /// Moves the latest editor session redo record back to the undo stack.
    pub fn mcp_redo(&mut self) -> McpUndoRedoResponse {
        let record = self.command_history_mut().pop_redo();
        self.mcp_undo_redo_response(record)
    }

    fn mcp_response_from_result(
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
            .command_history_mut()
            .commit_result(command.command_kind(), &command)
            .expect("MCP command summaries should be valid");

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
            dirty_state: dirty_state_response(&self.dirty_state()),
        }
    }

    fn mcp_undo_redo_response(&self, record: Option<UndoRedoRecord>) -> McpUndoRedoResponse {
        McpUndoRedoResponse {
            moved: record.is_some(),
            undo_group: record.as_ref().map(|record| record.group_id().raw()),
            summary: record.map(|record| record.summary().to_owned()),
            dirty_state: dirty_state_response(&self.dirty_state()),
        }
    }
}

fn mcp_validation_error(command_kind: &str, reason: impl Into<String>) -> CommandError {
    CommandError::ValidationFailed {
        command_kind: command_kind.to_owned(),
        reason: reason.into(),
    }
}

fn collect_instance_summary(
    scene: &Scene,
    instance_id: kinetik_core::InstanceId,
    summaries: &mut Vec<SceneInstanceSummary>,
) -> kinetik_scene::SceneResult<()> {
    let instance = scene.get(instance_id)?;
    let children = scene.children(instance_id)?.to_vec();
    summaries.push(SceneInstanceSummary {
        instance_id: instance.id.raw(),
        guid: instance.guid.raw(),
        class_name: instance.class_name.clone(),
        name: instance.name.clone(),
        scene_path: scene.path(instance_id)?,
        parent_id: instance.parent.map(kinetik_core::InstanceId::raw),
        child_ids: children.iter().map(|id| id.raw()).collect(),
    });

    for child_id in children {
        collect_instance_summary(scene, child_id, summaries)?;
    }

    Ok(())
}
