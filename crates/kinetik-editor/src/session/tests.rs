use kinetik_command::create_scene_child_instance;
use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};
use kinetik_project::{
    ProjectDocumentRefs, ProjectIdentity, ProjectModel, ProjectSettingsDocument,
};
use kinetik_scene::Scene;

use super::*;
use crate::EditorPanel;

fn demo_project() -> ProjectModel {
    ProjectModel::new(
        ProjectSettingsDocument::new(ProjectIdentity::new("Demo", "0.1").expect("valid identity")),
        ProjectDocumentRefs::default(),
    )
}

fn open_demo_session() -> EditorSession {
    let mut session = EditorSession::new();
    session.open_project(
        demo_project(),
        Scene::default_scene().expect("valid default scene"),
    );
    session
}

#[test]
fn session_opens_project_and_active_scene_without_editor_state_in_project() {
    let session = open_demo_session();

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
    let mut session = open_demo_session();
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
    let mut session = open_demo_session();
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
    let mut session = open_demo_session();
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

#[test]
fn explorer_snapshot_lists_default_scene_parent_before_child() {
    let session = open_demo_session();

    let snapshot = session.explorer_snapshot().expect("snapshot");

    assert_eq!(snapshot.rows()[0].scene_path, "/Game");
    assert_eq!(snapshot.rows()[1].scene_path, "/Game/Workspace");
    assert_eq!(snapshot.rows()[1].depth, 1);
}

#[test]
fn explorer_create_selects_new_instance_and_marks_dirty() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;

    let block = session
        .explorer_create_child(workspace, "Part", "Block")
        .expect("create through explorer");

    let snapshot = session.explorer_snapshot().expect("snapshot");
    assert_eq!(
        snapshot.row_by_id(block).unwrap().scene_path,
        "/Game/Workspace/Block"
    );
    assert!(matches!(
        session.selection().document(),
        EditorDocumentSelection::SceneInstance { id, .. } if *id == block
    ));
    assert_eq!(
        session.dirty_state().changes()[0].change_summary(),
        "created /Game/Workspace/Block"
    );
}

#[test]
fn explorer_rename_refreshes_selected_path() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    let scene = session.active_scene().unwrap().clone();
    session
        .selection_mut()
        .select_scene_instance(&scene, workspace)
        .unwrap();

    session
        .explorer_rename(workspace, "World")
        .expect("rename through explorer");

    assert!(matches!(
        session.selection().document(),
        EditorDocumentSelection::SceneInstance { scene_path, .. } if scene_path == "/Game/World"
    ));
}

#[test]
fn explorer_reparent_duplicate_and_delete_use_command_history() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    let scripts = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Scripts")
        .unwrap()
        .id;
    let folder = session
        .explorer_create_child(workspace, "Folder", "Folder")
        .expect("create folder");

    session
        .explorer_reparent(folder, scripts)
        .expect("reparent folder");
    let copy = session
        .explorer_duplicate(folder, scripts)
        .expect("duplicate folder");
    let deleted = session.explorer_delete(copy).expect("delete copy");

    assert_eq!(deleted, vec![copy]);
    assert_eq!(session.command_history().undo_stack().len(), 4);

    let undo = session.explorer_undo_stack_move().expect("undo record");
    assert_eq!(undo.summary(), "Delete Instance");
    assert_eq!(session.command_history().redo_stack().len(), 1);

    let redo = session.explorer_redo_stack_move().expect("redo record");
    assert_eq!(redo.summary(), "Delete Instance");
    assert!(session.command_history().redo_stack().is_empty());
}

#[test]
fn explorer_rejects_invalid_command_without_dirtying_session() {
    let mut session = open_demo_session();

    let error = session
        .explorer_delete(kinetik_core::InstanceId::new(999))
        .expect_err("invalid delete should fail");

    assert!(matches!(error, ExplorerCommandError::Command(_)));
    assert!(session.dirty_state().is_clean());
}
