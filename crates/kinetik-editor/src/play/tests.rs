use kinetik_app::{RuntimeInstanceId, RUNTIME_INSTANCE_ID_START};
use kinetik_command::CommandTargetMode;
use kinetik_project::{
    ProjectDocumentRefs, ProjectIdentity, ProjectModel, ProjectSettingsDocument,
};
use kinetik_scene::Scene;

use crate::{EditorModeState, EditorSession, McpPlayCommand};

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
fn play_mode_start_step_and_stop_own_runtime_sandbox() {
    let mut session = open_demo_session();
    let edit_document_before = session.active_scene().unwrap().to_document().unwrap();

    session.start_play_mode().expect("start play");

    let play = session.play_session().expect("play session");
    assert_eq!(session.mode(), EditorModeState::Play);
    assert_eq!(
        play.world().root_id(),
        Some(RuntimeInstanceId::new(RUNTIME_INSTANCE_ID_START))
    );
    assert_ne!(
        play.world().root_id().unwrap().raw(),
        session.active_scene().unwrap().root_id().unwrap().raw()
    );
    assert_eq!(
        play.world()
            .runtime_id_for_edit_guid(edit_document_before.root.guid),
        Some(RuntimeInstanceId::new(RUNTIME_INSTANCE_ID_START))
    );

    let step = session.step_play_mode(1.0 / 30.0).expect("step play");
    assert_eq!(step.frame_index, 1);
    assert!(step.fixed_steps >= 1);
    assert_eq!(
        session
            .play_session()
            .unwrap()
            .last_step()
            .unwrap()
            .frame_index,
        1
    );

    session.stop_play_mode();

    assert_eq!(session.mode(), EditorModeState::Edit);
    assert!(session.play_session().is_none());
    assert_eq!(
        session.active_scene().unwrap().to_document().unwrap(),
        edit_document_before
    );
    assert!(session.dirty_state().is_clean());
}

#[test]
fn play_runtime_mutations_do_not_persist_or_dirty_edit_scene() {
    let mut session = open_demo_session();
    let edit_document_before = session.active_scene().unwrap().to_document().unwrap();

    session.start_play_mode().expect("start play");
    let parent = session.play_session().unwrap().world().root_id().unwrap();
    let spawned = session
        .play_session_mut()
        .unwrap()
        .world_mut()
        .spawn_runtime_child(parent, "Part", "RuntimeOnly")
        .expect("runtime spawn");

    assert_eq!(
        session
            .play_session()
            .unwrap()
            .world()
            .get(spawned)
            .unwrap()
            .edit_guid,
        None
    );
    assert_eq!(
        session.active_scene().unwrap().to_document().unwrap(),
        edit_document_before
    );
    assert!(session.dirty_state().is_clean());

    session.stop_play_mode();

    assert!(session.play_session().is_none());
    assert_eq!(
        session.active_scene().unwrap().to_document().unwrap(),
        edit_document_before
    );
}

#[test]
fn play_diagnostics_are_visible_in_editor_and_mcp_snapshot() {
    let mut session = open_demo_session();

    session.start_play_mode().expect("start play");
    session.step_play_mode(0.0).expect("step play");

    let panel = session.diagnostics_panel();
    assert_eq!(panel.items().last().unwrap().code, "KT_RUNTIME_PLAY_STATE");
    assert_eq!(panel.items().last().unwrap().source, "Runtime");

    let snapshot = session.mcp_snapshot();
    assert!(snapshot.play_state.is_playing);
    assert_eq!(snapshot.play_state.frame_index, Some(1));
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "KT_RUNTIME_PLAY_STATE"));
}

#[test]
fn mcp_play_commands_enforce_edit_play_target_modes() {
    let mut session = open_demo_session();

    let ambiguous = session.mcp_execute_play_command(&McpPlayCommand::Start { target_mode: None });
    assert_eq!(
        ambiguous.diagnostics[0].code,
        "KT_COMMAND_AMBIGUOUS_TARGET_MODE"
    );
    assert!(!ambiguous.play_state.is_playing);

    let started = session.mcp_execute_play_command(&McpPlayCommand::Start {
        target_mode: Some(CommandTargetMode::Edit),
    });
    assert!(started.diagnostics.is_empty());
    assert!(started.play_state.is_playing);

    let wrong_step = session.mcp_execute_play_command(&McpPlayCommand::Step {
        target_mode: Some(CommandTargetMode::Edit),
        delta_seconds: 0.0,
    });
    assert_eq!(
        wrong_step.diagnostics[0].code,
        "KT_COMMAND_WRONG_TARGET_MODE"
    );

    let stepped = session.mcp_execute_play_command(&McpPlayCommand::Step {
        target_mode: Some(CommandTargetMode::Play),
        delta_seconds: 0.0,
    });
    assert!(stepped.diagnostics.is_empty());
    assert_eq!(stepped.play_state.frame_index, Some(1));

    let stop_response = session.mcp_execute_play_command(&McpPlayCommand::Stop {
        target_mode: Some(CommandTargetMode::Play),
    });
    assert!(stop_response.diagnostics.is_empty());
    assert!(!stop_response.play_state.is_playing);
}
