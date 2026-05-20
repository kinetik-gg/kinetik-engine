//! Editor document-session state for Kinetik Studio.

use kinetik_command::{CommandHistory, DirtyStateExplanation};
use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticSeverity, InstanceGuid, InstanceId,
};
use kinetik_project::ProjectModel;
use kinetik_scene::{Scene, SceneResult};

use crate::EditorPanel;

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
    project: Option<ProjectModel>,
    active_scene: Option<Scene>,
    selection: EditorSelection,
    command_history: CommandHistory,
    session_diagnostics: Vec<Diagnostic>,
    mode: EditorModeState,
}

impl EditorSession {
    /// Creates a closed editor session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens project and scene document state into the editor session.
    pub fn open_project(&mut self, project: ProjectModel, active_scene: Scene) {
        self.project = Some(project);
        self.active_scene = Some(active_scene);
        self.selection.clear();
        self.command_history = CommandHistory::new();
        self.session_diagnostics.clear();
        self.mode = EditorModeState::Edit;
    }

    /// Closes the current project and clears editor-only state.
    pub fn close_project(&mut self) {
        self.project = None;
        self.active_scene = None;
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

    /// Returns mutable active scene state for command execution.
    #[must_use]
    pub fn active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.active_scene.as_mut()
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
}

#[cfg(test)]
mod tests {
    use kinetik_command::create_scene_child_instance;
    use kinetik_core::{DiagnosticCode, DiagnosticSource};
    use kinetik_project::{ProjectDocumentRefs, ProjectIdentity, ProjectSettingsDocument};

    use super::*;

    fn demo_project() -> ProjectModel {
        ProjectModel::new(
            ProjectSettingsDocument::new(
                ProjectIdentity::new("Demo", "0.1").expect("valid identity"),
            ),
            ProjectDocumentRefs::default(),
        )
    }

    #[test]
    fn session_opens_project_and_active_scene_without_editor_state_in_project() {
        let mut session = EditorSession::new();
        session.open_project(
            demo_project(),
            Scene::default_scene().expect("valid default scene"),
        );

        let summary = session.project_summary().expect("open project");
        assert!(session.is_open());
        assert_eq!(summary.project_name(), "Demo");
        assert_eq!(summary.engine_compatibility(), "0.1");
        assert_eq!(summary.active_scene_path(), "scenes/main.knscene");
        assert_eq!(session.mode(), EditorModeState::Edit);
        assert!(matches!(
            session.selection().document(),
            EditorDocumentSelection::None
        ));
    }

    #[test]
    fn session_close_clears_editor_owned_state() {
        let mut session = EditorSession::new();
        session.open_project(
            demo_project(),
            Scene::default_scene().expect("valid default scene"),
        );
        session.selection_mut().focus_panel(EditorPanel::Explorer);
        session.replace_session_diagnostics([Diagnostic::new(
            DiagnosticCode::new("KT_TEST"),
            DiagnosticSeverity::Warning,
            DiagnosticSource::new("Test"),
            "warning",
        )]);
        session.enter_play_mode();

        session.close_project();

        assert!(!session.is_open());
        assert!(session.active_scene().is_none());
        assert_eq!(session.mode(), EditorModeState::Edit);
        assert!(session.diagnostics_panel().is_empty());
        assert!(matches!(
            session.selection().document(),
            EditorDocumentSelection::None
        ));
    }

    #[test]
    fn selection_tracks_scene_instance_identity_and_path() {
        let scene = Scene::default_scene().expect("valid default scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap();
        let mut selection = EditorSelection::new();

        selection
            .select_scene_instance(&scene, workspace.id)
            .expect("select workspace");
        selection.focus_panel(EditorPanel::Explorer);

        assert_eq!(selection.focus().panel(), Some(EditorPanel::Explorer));
        assert_eq!(
            selection.document(),
            &EditorDocumentSelection::SceneInstance {
                id: workspace.id,
                guid: workspace.guid,
                scene_path: "/Game/Workspace".to_owned(),
            }
        );
    }

    #[test]
    fn dirty_state_is_derived_from_command_history() {
        let mut session = EditorSession::new();
        session.open_project(
            demo_project(),
            Scene::default_scene().expect("valid default scene"),
        );
        let scene = session.active_scene_mut().expect("active scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        let command =
            create_scene_child_instance(scene, workspace, "Part", "Block", "scenes/main.knscene")
                .expect("create block");
        session
            .command_history_mut()
            .commit_result("Create Block", &command.command)
            .expect("commit")
            .expect("undo record");

        let dirty = session.dirty_state();

        assert!(!dirty.is_clean());
        assert_eq!(dirty.documents()[0].document_path(), "scenes/main.knscene");
        assert_eq!(
            dirty.changes()[0].change_summary(),
            "created /Game/Workspace/Block"
        );
    }

    #[test]
    fn diagnostics_panel_projects_session_diagnostics_in_order() {
        let mut session = EditorSession::new();
        session.open_project(
            demo_project(),
            Scene::default_scene().expect("valid default scene"),
        );
        session.replace_session_diagnostics([
            Diagnostic::new(
                DiagnosticCode::new("KT_FIRST"),
                DiagnosticSeverity::Info,
                DiagnosticSource::new("Test"),
                "first",
            ),
            Diagnostic::new(
                DiagnosticCode::new("KT_SECOND"),
                DiagnosticSeverity::Error,
                DiagnosticSource::new("Test"),
                "second",
            )
            .with_blocking_scope(DiagnosticBlockingScope::Save),
        ]);

        let panel = session.diagnostics_panel();

        assert_eq!(panel.items()[0].code, "KT_FIRST");
        assert_eq!(panel.items()[1].code, "KT_SECOND");
        assert_eq!(
            panel.items()[1].blocking,
            Some(DiagnosticBlockingScope::Save)
        );
    }

    #[test]
    fn mode_state_tracks_play_ownership_without_runtime_world() {
        let mut session = EditorSession::new();

        assert_eq!(session.mode(), EditorModeState::Edit);
        session.enter_play_mode();
        assert_eq!(session.mode(), EditorModeState::Play);
        session.stop_play_mode();
        assert_eq!(session.mode(), EditorModeState::Edit);
    }
}
