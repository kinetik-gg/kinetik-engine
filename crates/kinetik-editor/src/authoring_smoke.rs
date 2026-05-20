use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use kinetik_command::{CommandStatus, CommandTargetMode};
use kinetik_core::Vec3;
use kinetik_project::{
    ProjectDocumentRefs, ProjectIdentity, ProjectModel, ProjectSettingsDocument,
};
use kinetik_reflect::PropertyValue;
use kinetik_scene::{InstanceClassRegistry, Scene};

use crate::{
    EditorSession, McpPlayCommand, McpSceneMutationRequest, McpSelectionRequest,
    ViewportPickRequest,
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

#[test]
fn headless_3d_authoring_loop_saves_reloads_and_runs_play_mode() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;
    let part = session
        .explorer_create_child(workspace, "Part", "AuthoringBlock")
        .expect("create authoring block");

    session
        .inspector_set_property(
            part,
            "Transform.Position",
            PropertyValue::Vec3(Vec3::new(3.0, 1.5, -2.0)),
        )
        .expect("set position");
    session
        .inspector_set_property(
            part,
            "Transform.Scale",
            PropertyValue::Vec3(Vec3::new(2.0, 1.0, 4.0)),
        )
        .expect("set scale");
    session
        .inspector_set_property(part, "Visible", PropertyValue::Bool(true))
        .expect("set render-facing visibility");

    let scene = session.active_scene().unwrap().clone();
    session
        .selection_mut()
        .select_scene_instance(&scene, part)
        .expect("select part");
    session
        .viewport_focus_selected()
        .expect("focus authored part");

    let root = temp_project_root("m24-headless-authoring-loop");
    session.save_project_to(&root).expect("save project");
    assert!(session.dirty_state().is_clean());

    let mut reloaded = EditorSession::new();
    reloaded
        .reload_project_from(&root, default_scene_registry())
        .expect("reload project");
    let reloaded_part = reloaded
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace/AuthoringBlock")
        .unwrap()
        .id;

    assert_eq!(
        reloaded
            .active_scene()
            .unwrap()
            .get_property(reloaded_part, "Transform.Position")
            .unwrap(),
        &PropertyValue::Vec3(Vec3::new(3.0, 1.5, -2.0))
    );
    assert_eq!(
        reloaded
            .active_scene()
            .unwrap()
            .world_bounds(reloaded_part)
            .unwrap()
            .size(),
        Vec3::new(2.0, 1.0, 4.0)
    );

    reloaded.start_play_mode().expect("start play mode");
    let step = reloaded.step_play_mode(1.0 / 60.0).expect("step play mode");
    assert_eq!(step.frame_index, 1);
    assert!(reloaded
        .diagnostics_panel()
        .items()
        .iter()
        .any(|item| item.code == "KT_RUNTIME_PLAY_STATE"));
    reloaded.stop_play_mode();
    assert!(reloaded.dirty_state().is_clean());

    std::fs::remove_dir_all(root).expect("cleanup temp project");
}

#[test]
fn mcp_driven_3d_authoring_smoke_preserves_scene_data_through_play() {
    let mut session = open_demo_session();
    let workspace = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace")
        .unwrap()
        .id;

    let create = session.mcp_execute_scene_mutation(McpSceneMutationRequest::CreateInstance {
        target_mode: Some(CommandTargetMode::Edit),
        parent_id: workspace,
        class_name: "Part".to_owned(),
        name: "McpBlock".to_owned(),
    });
    assert_eq!(create.status, CommandStatus::Succeeded);
    let part = session
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace/McpBlock")
        .unwrap()
        .id;

    let position = session.mcp_execute_scene_mutation(McpSceneMutationRequest::SetProperty {
        target_mode: Some(CommandTargetMode::Edit),
        instance_id: part,
        property_path: "Transform.Position".to_owned(),
        value: PropertyValue::Vec3(Vec3::new(-2.0, 4.0, 6.0)),
    });
    assert_eq!(position.status, CommandStatus::Succeeded);
    let visible = session.mcp_execute_scene_mutation(McpSceneMutationRequest::SetProperty {
        target_mode: Some(CommandTargetMode::Edit),
        instance_id: part,
        property_path: "Visible".to_owned(),
        value: PropertyValue::Bool(false),
    });
    assert_eq!(visible.status, CommandStatus::Succeeded);

    session.mcp_apply_selection(McpSelectionRequest::SelectSceneInstance { instance_id: part });
    let focus = session.mcp_viewport_focus_selected();
    assert!(focus.diagnostics.is_empty());
    let pick =
        session.mcp_viewport_pick(ViewportPickRequest::new(kinetik_core::Vec2::new(8.0, 8.0)));
    assert!(pick.pick.instance_id.is_none());

    let root = temp_project_root("m24-mcp-authoring-loop");
    session
        .save_project_to(&root)
        .expect("save MCP-authored project");
    let mut reloaded = EditorSession::new();
    reloaded
        .reload_project_from(&root, default_scene_registry())
        .expect("reload MCP-authored project");
    let reloaded_part = reloaded
        .active_scene()
        .unwrap()
        .get_by_path("/Game/Workspace/McpBlock")
        .unwrap()
        .id;

    assert_eq!(
        reloaded
            .active_scene()
            .unwrap()
            .get_property(reloaded_part, "Transform.Position")
            .unwrap(),
        &PropertyValue::Vec3(Vec3::new(-2.0, 4.0, 6.0))
    );
    assert_eq!(
        reloaded
            .active_scene()
            .unwrap()
            .get_property(reloaded_part, "Visible")
            .unwrap(),
        &PropertyValue::Bool(false)
    );

    let start = reloaded.mcp_execute_play_command(&McpPlayCommand::Start {
        target_mode: Some(CommandTargetMode::Edit),
    });
    assert!(start.diagnostics.is_empty());
    let step_response = reloaded.mcp_execute_play_command(&McpPlayCommand::Step {
        target_mode: Some(CommandTargetMode::Play),
        delta_seconds: 0.0,
    });
    assert_eq!(step_response.play_state.frame_index, Some(1));
    let snapshot = reloaded.mcp_snapshot();
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "KT_RUNTIME_PLAY_STATE"));
    let halt_result = reloaded.mcp_execute_play_command(&McpPlayCommand::Stop {
        target_mode: Some(CommandTargetMode::Play),
    });
    assert!(halt_result.diagnostics.is_empty());

    std::fs::remove_dir_all(root).expect("cleanup temp project");
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

fn default_scene_registry() -> InstanceClassRegistry {
    InstanceClassRegistry::with_default_scene_classes().expect("built-in scene classes")
}
