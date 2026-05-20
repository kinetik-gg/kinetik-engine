//! Editor document-session state for Kinetik Studio.

use std::fmt;

use kinetik_command::{
    create_scene_child_instance, delete_scene_instance, duplicate_scene_instance,
    rename_scene_instance, reparent_scene_instance, CommandError, CommandHistory,
    DirtyStateExplanation, UndoRedoRecord,
};
use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticSeverity, InstanceGuid, InstanceId,
};
use kinetik_project::ProjectModel;
use kinetik_resource::AssetManifest;
use kinetik_scene::{Scene, SceneResult};

use crate::{EditorPanel, ExplorerSnapshot};

/// Lightweight project/session view exposed by the editor session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorSessionProject {
    project_name: String,
    engine_compatibility: String,
    active_scene_path: String,
    assets_manifest_path: String,
    instances_manifest_path: String,
}

impl EditorSessionProject {
    fn from_project(project: &ProjectModel) -> Self {
        Self {
            project_name: project.settings().identity().name().to_owned(),
            engine_compatibility: project
                .settings()
                .identity()
                .engine_compatibility()
                .to_owned(),
            active_scene_path: project.documents().active_scene().to_owned(),
            assets_manifest_path: project.documents().assets_manifest().to_owned(),
            instances_manifest_path: project.documents().instances_manifest().to_owned(),
        }
    }

    /// Returns the project display name.
    #[must_use]
    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    /// Returns the engine compatibility string.
    #[must_use]
    pub fn engine_compatibility(&self) -> &str {
        &self.engine_compatibility
    }

    /// Returns the active scene document path.
    #[must_use]
    pub fn active_scene_path(&self) -> &str {
        &self.active_scene_path
    }

    /// Returns the asset manifest document path.
    #[must_use]
    pub fn assets_manifest_path(&self) -> &str {
        &self.assets_manifest_path
    }

    /// Returns the instance manifest document path.
    #[must_use]
    pub fn instances_manifest_path(&self) -> &str {
        &self.instances_manifest_path
    }
}

/// Editor-selected document or scene object.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum EditorDocumentSelection {
    /// No document or object is selected.
    #[default]
    None,
    /// A scene instance is selected in edit state.
    SceneInstance {
        /// Runtime/edit instance ID for the active scene document.
        id: InstanceId,
        /// Stable edit-world instance GUID.
        guid: InstanceGuid,
        /// Human-readable scene path at selection time.
        scene_path: String,
    },
    /// A project document is selected.
    ProjectDocument {
        /// Workspace-relative document path.
        path: String,
    },
}

/// Current editor focus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorFocus {
    panel: Option<EditorPanel>,
}

impl EditorFocus {
    /// Creates an empty focus state.
    #[must_use]
    pub const fn new() -> Self {
        Self { panel: None }
    }

    /// Returns the focused panel.
    #[must_use]
    pub const fn panel(&self) -> Option<EditorPanel> {
        self.panel
    }

    /// Sets the focused panel.
    pub fn set_panel(&mut self, panel: EditorPanel) {
        self.panel = Some(panel);
    }

    /// Clears focus.
    pub fn clear(&mut self) {
        self.panel = None;
    }
}

impl Default for EditorFocus {
    fn default() -> Self {
        Self::new()
    }
}

/// Editor selection and focus state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EditorSelection {
    document: EditorDocumentSelection,
    focus: EditorFocus,
}

impl EditorSelection {
    /// Creates an empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the selected document/object.
    #[must_use]
    pub const fn document(&self) -> &EditorDocumentSelection {
        &self.document
    }

    /// Returns current focus.
    #[must_use]
    pub const fn focus(&self) -> &EditorFocus {
        &self.focus
    }

    /// Selects a project document path.
    pub fn select_project_document(&mut self, path: impl Into<String>) {
        self.document = EditorDocumentSelection::ProjectDocument { path: path.into() };
    }

    /// Selects a scene instance from the active scene.
    ///
    /// # Errors
    ///
    /// Returns [`kinetik_scene::SceneError`] when the instance ID does not exist.
    pub fn select_scene_instance(&mut self, scene: &Scene, id: InstanceId) -> SceneResult<()> {
        let instance = scene.get(id)?;
        self.document = EditorDocumentSelection::SceneInstance {
            id,
            guid: instance.guid,
            scene_path: scene.path(id)?,
        };
        Ok(())
    }

    /// Sets the focused panel.
    pub fn focus_panel(&mut self, panel: EditorPanel) {
        self.focus.set_panel(panel);
    }

    /// Clears selection and focus.
    pub fn clear(&mut self) {
        self.document = EditorDocumentSelection::None;
        self.focus.clear();
    }
}

/// Edit/play ownership state for the editor session.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EditorModeState {
    /// Editing source project state.
    #[default]
    Edit,
    /// A play-world owner exists. Runtime world behavior lands in M22.
    Play,
}

/// One projected diagnostic row for the diagnostics panel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticPanelItem {
    /// Stable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// Optional blocking scope.
    pub blocking: Option<DiagnosticBlockingScope>,
    /// Source system name.
    pub source: String,
    /// Human-readable message.
    pub message: String,
}

impl DiagnosticPanelItem {
    fn from_diagnostic(diagnostic: &Diagnostic) -> Self {
        Self {
            code: diagnostic.code.as_str().to_owned(),
            severity: diagnostic.severity,
            blocking: diagnostic.blocking,
            source: diagnostic.source.as_str().to_owned(),
            message: diagnostic.message.clone(),
        }
    }
}

/// Diagnostics panel state derived from structured diagnostics.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticsPanelState {
    items: Vec<DiagnosticPanelItem>,
}

impl DiagnosticsPanelState {
    /// Creates diagnostics panel state from current diagnostics.
    #[must_use]
    pub fn from_diagnostics<'a, I>(diagnostics: I) -> Self
    where
        I: IntoIterator<Item = &'a Diagnostic>,
    {
        Self {
            items: diagnostics
                .into_iter()
                .map(DiagnosticPanelItem::from_diagnostic)
                .collect(),
        }
    }

    /// Returns projected diagnostics in deterministic order.
    #[must_use]
    pub fn items(&self) -> &[DiagnosticPanelItem] {
        &self.items
    }

    /// Returns whether the diagnostics panel has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Open Kinetik Studio editor session state.
#[derive(Debug, Default)]
pub struct EditorSession {
    pub(crate) project: Option<ProjectModel>,
    pub(crate) active_scene: Option<Scene>,
    pub(crate) asset_manifest: AssetManifest,
    pub(crate) selection: EditorSelection,
    pub(crate) command_history: CommandHistory,
    pub(crate) session_diagnostics: Vec<Diagnostic>,
    pub(crate) mode: EditorModeState,
}

/// Error returned by command-backed Explorer actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExplorerCommandError {
    /// No project/scene is open in the editor session.
    NoActiveScene,
    /// Existing command validation rejected the action.
    Command(CommandError),
    /// Scene hierarchy projection failed after mutation.
    Scene(String),
}

impl fmt::Display for ExplorerCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveScene => formatter.write_str("no active scene is open"),
            Self::Command(error) => write!(formatter, "{error}"),
            Self::Scene(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for ExplorerCommandError {}

impl From<CommandError> for ExplorerCommandError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

impl EditorSession {
    /// Creates a closed editor session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens project and scene document state into the editor session.
    pub fn open_project(&mut self, project: ProjectModel, active_scene: Scene) {
        self.open_project_with_assets(project, active_scene, AssetManifest::new());
    }

    /// Opens project, scene, and asset manifest document state into the editor session.
    pub fn open_project_with_assets(
        &mut self,
        project: ProjectModel,
        active_scene: Scene,
        asset_manifest: AssetManifest,
    ) {
        self.project = Some(project);
        self.active_scene = Some(active_scene);
        self.asset_manifest = asset_manifest;
        self.selection.clear();
        self.command_history = CommandHistory::new();
        self.session_diagnostics.clear();
        self.mode = EditorModeState::Edit;
    }

    /// Closes the current project and clears editor-only state.
    pub fn close_project(&mut self) {
        self.project = None;
        self.active_scene = None;
        self.asset_manifest = AssetManifest::new();
        self.selection.clear();
        self.command_history = CommandHistory::new();
        self.session_diagnostics.clear();
        self.mode = EditorModeState::Edit;
    }

    /// Returns whether a project is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.project.is_some()
    }

    /// Returns a lightweight project summary for the open session.
    #[must_use]
    pub fn project_summary(&self) -> Option<EditorSessionProject> {
        self.project
            .as_ref()
            .map(EditorSessionProject::from_project)
    }

    /// Returns the open project model.
    #[must_use]
    pub const fn project(&self) -> Option<&ProjectModel> {
        self.project.as_ref()
    }

    /// Returns the active scene.
    #[must_use]
    pub const fn active_scene(&self) -> Option<&Scene> {
        self.active_scene.as_ref()
    }

    /// Returns the active project asset manifest.
    #[must_use]
    pub const fn asset_manifest(&self) -> &AssetManifest {
        &self.asset_manifest
    }

    /// Returns mutable active scene state for command execution.
    #[must_use]
    pub fn active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.active_scene.as_mut()
    }

    /// Returns an Explorer snapshot for the active scene.
    ///
    /// # Errors
    ///
    /// Returns [`ExplorerCommandError`] when no scene is open or hierarchy
    /// projection fails.
    pub fn explorer_snapshot(&self) -> Result<ExplorerSnapshot, ExplorerCommandError> {
        let scene = self
            .active_scene
            .as_ref()
            .ok_or(ExplorerCommandError::NoActiveScene)?;
        ExplorerSnapshot::from_scene(scene)
            .map_err(|error| ExplorerCommandError::Scene(error.to_string()))
    }

    /// Returns current selection and focus state.
    #[must_use]
    pub const fn selection(&self) -> &EditorSelection {
        &self.selection
    }

    /// Returns mutable selection and focus state.
    #[must_use]
    pub fn selection_mut(&mut self) -> &mut EditorSelection {
        &mut self.selection
    }

    /// Returns command history.
    #[must_use]
    pub const fn command_history(&self) -> &CommandHistory {
        &self.command_history
    }

    /// Returns mutable command history for command integration.
    #[must_use]
    pub fn command_history_mut(&mut self) -> &mut CommandHistory {
        &mut self.command_history
    }

    /// Creates a child instance through the shared scene command path.
    ///
    /// # Errors
    ///
    /// Returns [`ExplorerCommandError`] when no scene is open, validation fails,
    /// or selection cannot be updated after mutation.
    pub fn explorer_create_child(
        &mut self,
        parent_id: InstanceId,
        class_name: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<InstanceId, ExplorerCommandError> {
        let document_path = self.active_scene_document_path()?;
        let result = create_scene_child_instance(
            self.active_scene_mut_or_error()?,
            parent_id,
            class_name,
            name,
            document_path,
        )?;
        self.commit_explorer_command("Create Instance", &result.command)?;
        self.select_active_scene_instance(result.instance_id)?;
        Ok(result.instance_id)
    }

    /// Deletes a scene instance through the shared scene command path.
    ///
    /// # Errors
    ///
    /// Returns [`ExplorerCommandError`] when no scene is open or validation
    /// fails.
    pub fn explorer_delete(
        &mut self,
        instance_id: InstanceId,
    ) -> Result<Vec<InstanceId>, ExplorerCommandError> {
        let document_path = self.active_scene_document_path()?;
        let result = delete_scene_instance(
            self.active_scene_mut_or_error()?,
            instance_id,
            document_path,
        )?;
        self.commit_explorer_command("Delete Instance", &result.command)?;
        if self.selection_references_any(&result.deleted_ids) {
            self.selection.clear();
        }
        Ok(result.deleted_ids)
    }

    /// Renames a scene instance through the shared scene command path.
    ///
    /// # Errors
    ///
    /// Returns [`ExplorerCommandError`] when no scene is open, validation fails,
    /// or selection cannot be refreshed after mutation.
    pub fn explorer_rename(
        &mut self,
        instance_id: InstanceId,
        new_name: impl Into<String>,
    ) -> Result<(), ExplorerCommandError> {
        let document_path = self.active_scene_document_path()?;
        let command = rename_scene_instance(
            self.active_scene_mut_or_error()?,
            instance_id,
            new_name,
            document_path,
        )?;
        self.commit_explorer_command("Rename Instance", &command)?;
        self.refresh_selection_if_selected(instance_id)?;
        Ok(())
    }

    /// Duplicates a scene instance through the shared scene command path.
    ///
    /// # Errors
    ///
    /// Returns [`ExplorerCommandError`] when no scene is open, validation fails,
    /// or selection cannot be updated after mutation.
    pub fn explorer_duplicate(
        &mut self,
        instance_id: InstanceId,
        new_parent: InstanceId,
    ) -> Result<InstanceId, ExplorerCommandError> {
        let document_path = self.active_scene_document_path()?;
        let result = duplicate_scene_instance(
            self.active_scene_mut_or_error()?,
            instance_id,
            new_parent,
            document_path,
        )?;
        self.commit_explorer_command("Duplicate Instance", &result.command)?;
        self.select_active_scene_instance(result.new_root_id)?;
        Ok(result.new_root_id)
    }

    /// Reparents a scene instance through the shared scene command path.
    ///
    /// # Errors
    ///
    /// Returns [`ExplorerCommandError`] when no scene is open, validation fails,
    /// or selection cannot be refreshed after mutation.
    pub fn explorer_reparent(
        &mut self,
        instance_id: InstanceId,
        new_parent: InstanceId,
    ) -> Result<(), ExplorerCommandError> {
        let document_path = self.active_scene_document_path()?;
        let result = reparent_scene_instance(
            self.active_scene_mut_or_error()?,
            instance_id,
            new_parent,
            document_path,
        )?;
        self.commit_explorer_command("Reparent Instance", &result.command)?;
        self.refresh_selection_if_selected(result.instance_id)?;
        Ok(())
    }

    /// Moves the latest Explorer command record to the redo stack.
    #[must_use]
    pub fn explorer_undo_stack_move(&mut self) -> Option<UndoRedoRecord> {
        self.command_history.pop_undo()
    }

    /// Moves the latest Explorer redo record back to the undo stack.
    #[must_use]
    pub fn explorer_redo_stack_move(&mut self) -> Option<UndoRedoRecord> {
        self.command_history.pop_redo()
    }

    /// Returns dirty-state explanation derived from command history.
    #[must_use]
    pub fn dirty_state(&self) -> DirtyStateExplanation {
        DirtyStateExplanation::from_history(&self.command_history)
    }

    /// Replaces editor-session-owned diagnostics.
    pub fn replace_session_diagnostics<I>(&mut self, diagnostics: I)
    where
        I: IntoIterator<Item = Diagnostic>,
    {
        self.session_diagnostics = diagnostics.into_iter().collect();
    }

    /// Returns diagnostics panel state from project and session diagnostics.
    #[must_use]
    pub fn diagnostics_panel(&self) -> DiagnosticsPanelState {
        let project_diagnostics = self
            .project
            .as_ref()
            .into_iter()
            .flat_map(|project| project.diagnostics().diagnostics());
        let session_diagnostics = self.session_diagnostics.iter();
        DiagnosticsPanelState::from_diagnostics(project_diagnostics.chain(session_diagnostics))
    }

    /// Returns current edit/play mode ownership.
    #[must_use]
    pub const fn mode(&self) -> EditorModeState {
        self.mode
    }

    /// Marks the editor session as owning play mode.
    pub fn enter_play_mode(&mut self) {
        self.mode = EditorModeState::Play;
    }

    /// Returns ownership to edit mode.
    pub fn stop_play_mode(&mut self) {
        self.mode = EditorModeState::Edit;
    }

    pub(crate) fn active_scene_document_path(&self) -> Result<String, ExplorerCommandError> {
        let project = self
            .project
            .as_ref()
            .ok_or(ExplorerCommandError::NoActiveScene)?;
        Ok(project.documents().active_scene().to_owned())
    }

    fn active_scene_mut_or_error(&mut self) -> Result<&mut Scene, ExplorerCommandError> {
        self.active_scene
            .as_mut()
            .ok_or(ExplorerCommandError::NoActiveScene)
    }

    fn commit_explorer_command(
        &mut self,
        summary: &str,
        command: &kinetik_command::CommandResult,
    ) -> Result<(), ExplorerCommandError> {
        self.command_history
            .commit_result(summary, command)
            .map_err(ExplorerCommandError::Command)?;
        Ok(())
    }

    fn select_active_scene_instance(&mut self, id: InstanceId) -> Result<(), ExplorerCommandError> {
        let scene = self
            .active_scene
            .as_ref()
            .ok_or(ExplorerCommandError::NoActiveScene)?;
        self.selection
            .select_scene_instance(scene, id)
            .map_err(|error| ExplorerCommandError::Scene(error.to_string()))
    }

    pub(crate) fn refresh_selection_if_selected(
        &mut self,
        id: InstanceId,
    ) -> Result<(), ExplorerCommandError> {
        if matches!(
            self.selection.document(),
            EditorDocumentSelection::SceneInstance { id: selected, .. } if *selected == id
        ) {
            self.select_active_scene_instance(id)?;
        }
        Ok(())
    }

    fn selection_references_any(&self, ids: &[InstanceId]) -> bool {
        match self.selection.document() {
            EditorDocumentSelection::SceneInstance { id, .. } => ids.contains(id),
            EditorDocumentSelection::None | EditorDocumentSelection::ProjectDocument { .. } => {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests;
