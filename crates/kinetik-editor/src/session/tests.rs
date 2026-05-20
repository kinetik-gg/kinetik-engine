use kinetik_command::{create_scene_child_instance, CommandTargetMode};
use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};
use kinetik_project::{
    ProjectDocumentRefs, ProjectIdentity, ProjectModel, ProjectSettingsDocument,
};
use kinetik_reflect::{EditorEditability, PropertyType, PropertyValue};
use kinetik_resource::{AssetGuid, AssetManifest, AssetManifestEntry};
use kinetik_scene::{InstanceClassRegistry, Scene};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::{
    EditorPanel, InspectorCommandError, McpSceneMutationRequest, McpSelectionRequest,
    McpSelectionResponse,
};

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

fn temp_project_root(test_name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    path.push(format!("kinetik-{test_name}-{unique}"));
    path
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
fn save_project_writes_golden_files_and_clears_dirty_state() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    session
        .explorer_create_child(workspace, "Part", "Block")
        .expect("create block");
    let manifest = AssetManifest::from_entries(vec![AssetManifestEntry::from_parts(
        AssetGuid::new(1),
        "res://assets/models/block.glb",
        "gltf",
        "1.0.0",
        "hash-block",
    )
    .unwrap()])
    .unwrap();
    let project = session.project().unwrap().clone();
    let scene = session.active_scene().unwrap().clone();
    session.open_project_with_assets(project, scene, manifest);
    session
        .explorer_create_child(workspace, "Part", "SecondBlock")
        .expect("make session dirty");

    let root = temp_project_root("save-golden");
    session.save_project_to(&root).expect("save project");

    assert!(session.dirty_state().is_clean());
    assert_eq!(
        std::fs::read_to_string(root.join("Kinetik.toml")).unwrap(),
        include_str!("../../fixtures/save_reload/Kinetik.toml")
    );
    assert_eq!(
        std::fs::read_to_string(root.join("project/assets.knmanifest")).unwrap(),
        include_str!("../../fixtures/save_reload/assets.knmanifest")
    );
    assert_eq!(
        std::fs::read_to_string(root.join("scenes/main.knscene")).unwrap(),
        include_str!("../../fixtures/save_reload/main.knscene")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reload_project_reconstructs_saved_scene_and_manifest() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    session
        .explorer_create_child(workspace, "Part", "Block")
        .expect("create block");
    let manifest = AssetManifest::from_entries(vec![AssetManifestEntry::from_parts(
        AssetGuid::new(1),
        "res://assets/models/block.glb",
        "gltf",
        "1.0.0",
        "hash-block",
    )
    .unwrap()])
    .unwrap();
    let project = session.project().unwrap().clone();
    let scene = session.active_scene().unwrap().clone();
    session.open_project_with_assets(project, scene, manifest.clone());

    let root = temp_project_root("reload-smoke");
    session.save_project_to(&root).expect("save project");

    let mut reloaded = EditorSession::new();
    reloaded
        .reload_project_from(
            &root,
            InstanceClassRegistry::with_default_scene_classes().unwrap(),
        )
        .expect("reload project");

    assert_eq!(
        reloaded.project().unwrap().settings(),
        session.project().unwrap().settings()
    );
    assert_eq!(
        reloaded.active_scene().unwrap().to_document().unwrap(),
        session.active_scene().unwrap().to_document().unwrap()
    );
    assert_eq!(reloaded.asset_manifest(), &manifest);
    assert!(reloaded.dirty_state().is_clean());
    assert!(matches!(
        reloaded.selection().document(),
        EditorDocumentSelection::None
    ));
    std::fs::remove_dir_all(root).unwrap();
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
fn mcp_snapshot_reports_editor_session_selection_focus_diagnostics_and_dirty_state() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    session
        .explorer_create_child(workspace, "Part", "Block")
        .expect("create block");
    let block = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace/Block")
        .unwrap()
        .id;
    session.mcp_apply_selection(McpSelectionRequest::SelectSceneInstance { instance_id: block });
    session.mcp_apply_selection(McpSelectionRequest::FocusPanel {
        panel: EditorPanel::Inspector,
    });
    session.replace_session_diagnostics([Diagnostic::new(
        DiagnosticCode::new("KT_TEST_MCP"),
        DiagnosticSeverity::Info,
        DiagnosticSource::new("Test"),
        "mcp parity",
    )]);

    let snapshot = session.mcp_snapshot();

    assert_eq!(snapshot.project.unwrap().project_name, "Demo");
    assert!(snapshot
        .scene
        .iter()
        .any(|instance| instance.scene_path == "/Game/Workspace/Block"));
    assert_eq!(
        snapshot.selection,
        McpSelectionResponse::SceneInstance {
            instance_id: block.raw(),
            guid: session
                .active_scene()
                .unwrap()
                .get(block)
                .unwrap()
                .guid
                .raw(),
            scene_path: "/Game/Workspace/Block".to_owned(),
        }
    );
    assert_eq!(snapshot.focus, Some(EditorPanel::Inspector));
    assert_eq!(snapshot.diagnostics[0].code, "KT_TEST_MCP");
    assert_eq!(
        snapshot.dirty_state.summaries,
        vec!["created /Game/Workspace/Block"]
    );
}

#[test]
fn mcp_selection_commands_update_same_session_state_as_ui_selection() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;

    let response = session.mcp_apply_selection(McpSelectionRequest::SelectSceneInstance {
        instance_id: workspace,
    });
    session.mcp_apply_selection(McpSelectionRequest::FocusPanel {
        panel: EditorPanel::Explorer,
    });

    assert!(response.diagnostics.is_empty());
    assert!(matches!(
        session.selection().document(),
        EditorDocumentSelection::SceneInstance { id, .. } if *id == workspace
    ));
    assert_eq!(
        session.selection().focus().panel(),
        Some(EditorPanel::Explorer)
    );

    let response = session.mcp_apply_selection(McpSelectionRequest::SelectProjectDocument {
        path: "Kinetik.toml".to_owned(),
    });
    assert_eq!(
        response.selection,
        McpSelectionResponse::ProjectDocument {
            path: "Kinetik.toml".to_owned(),
        }
    );

    let response = session.mcp_apply_selection(McpSelectionRequest::Clear);
    assert_eq!(response.selection, McpSelectionResponse::None);
    assert_eq!(response.focus, None);
}

#[test]
fn mcp_scene_mutation_matches_explorer_command_path() {
    let mut ui_session = open_demo_session();
    let mut mcp_session = open_demo_session();
    let workspace = ui_session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;

    let ui_block = ui_session
        .explorer_create_child(workspace, "Part", "Block")
        .expect("ui create");
    let response =
        mcp_session.mcp_execute_scene_mutation(McpSceneMutationRequest::CreateInstance {
            target_mode: Some(CommandTargetMode::Edit),
            parent_id: workspace,
            class_name: "Part".to_owned(),
            name: "Block".to_owned(),
        });

    assert!(response.diagnostics.is_empty());
    assert_eq!(
        response.change_summaries,
        vec!["created /Game/Workspace/Block"]
    );
    assert_eq!(response.undo_group, Some(1));
    assert_eq!(
        mcp_session.active_scene().unwrap().to_document().unwrap(),
        ui_session.active_scene().unwrap().to_document().unwrap()
    );
    assert_eq!(
        mcp_session
            .dirty_state()
            .changes()
            .iter()
            .map(kinetik_command::DirtyChangeExplanation::change_summary)
            .collect::<Vec<_>>(),
        ui_session
            .dirty_state()
            .changes()
            .iter()
            .map(kinetik_command::DirtyChangeExplanation::change_summary)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        mcp_session
            .active_scene()
            .unwrap()
            .get_by_path("/Game/Workspace/Block")
            .unwrap()
            .guid,
        ui_session
            .active_scene()
            .unwrap()
            .get(ui_block)
            .unwrap()
            .guid
    );
}

#[test]
fn mcp_scene_mutation_reports_ambiguous_mode_without_dirtying_session() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;

    let response = session.mcp_execute_scene_mutation(McpSceneMutationRequest::CreateInstance {
        target_mode: None,
        parent_id: workspace,
        class_name: "Part".to_owned(),
        name: "Block".to_owned(),
    });

    assert_eq!(
        response.diagnostics[0].code,
        "KT_COMMAND_AMBIGUOUS_TARGET_MODE"
    );
    assert!(response.dirty_state.summaries.is_empty());
    assert!(session.dirty_state().is_clean());
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

#[test]
fn inspector_snapshot_uses_reflection_descriptors_and_current_values() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    let part = session
        .explorer_create_child(workspace, "Part", "Block")
        .expect("create part");

    let inspector = session.inspector_snapshot(part).expect("inspector");
    let name = inspector.row("Name").expect("name row");
    let visible = inspector.row("Visible").expect("visible row");
    let position = inspector.row("Transform.Position").expect("position row");

    assert_eq!(inspector.scene_path, "/Game/Workspace/Block");
    assert_eq!(inspector.class_name, "Part");
    assert_eq!(name.display_name, "Name");
    assert_eq!(name.value_type, PropertyType::String);
    assert_eq!(name.value, PropertyValue::String("Block".to_owned()));
    assert_eq!(visible.value, PropertyValue::Bool(true));
    assert_eq!(position.editability, EditorEditability::Editable);
}

#[test]
fn inspector_set_selected_property_uses_command_path_and_refreshes_selection() {
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
        .inspector_set_selected_property("Name", PropertyValue::String("World".to_owned()))
        .expect("set selected name");

    assert_eq!(
        session.active_scene().unwrap().path(workspace).unwrap(),
        "/Game/World"
    );
    assert!(matches!(
        session.selection().document(),
        EditorDocumentSelection::SceneInstance { scene_path, .. } if scene_path == "/Game/World"
    ));
    assert_eq!(
        session.dirty_state().changes()[0].change_summary(),
        "set /Game/Workspace.Name"
    );
}

#[test]
fn inspector_rejects_invalid_property_value_without_dirtying_session() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    let part = session
        .explorer_create_child(workspace, "Part", "Block")
        .expect("create part");
    let dirty_before = session.dirty_state().changes().len();

    let error = session
        .inspector_set_property(part, "Visible", PropertyValue::String("yes".to_owned()))
        .expect_err("invalid type should fail");

    assert!(matches!(error, InspectorCommandError::Command(_)));
    assert_eq!(session.dirty_state().changes().len(), dirty_before);
    assert_eq!(
        session
            .active_scene()
            .unwrap()
            .get_property(part, "Visible")
            .unwrap(),
        &PropertyValue::Bool(true)
    );
}

#[test]
fn selected_inspector_requires_scene_instance_selection() {
    let session = open_demo_session();

    let error = session
        .selected_inspector_snapshot()
        .expect_err("no scene instance selected");

    assert_eq!(error, InspectorCommandError::NoSelectedInstance);
}
