use kinetik_core::{Vec2, Vec3};
use kinetik_project::{
    ProjectDocumentRefs, ProjectIdentity, ProjectModel, ProjectSettingsDocument,
};
use kinetik_reflect::PropertyValue;
use kinetik_scene::Scene;

use super::*;
use crate::{EditorSession, McpSelectionRequest, ViewportPickStatus, ViewportPickingContract};

const EPSILON: f32 = 0.0001;

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

fn add_positioned_part(scene: &mut Scene) -> InstanceId {
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let part = scene.add_child(workspace, "Part", "Block").unwrap();
    scene
        .set_property(
            part,
            "Transform.Position",
            PropertyValue::Vec3(Vec3::new(4.0, 2.0, -1.0)),
        )
        .unwrap();
    scene
        .set_property(
            part,
            "Transform.Scale",
            PropertyValue::Vec3(Vec3::new(2.0, 4.0, 6.0)),
        )
        .unwrap();
    part
}

#[test]
fn viewport_navigation_state_updates_without_scene_mutation() {
    let mut viewport = ViewportState::new();

    viewport.resize(Vec2::new(640.0, 480.0)).unwrap();
    viewport.orbit(0.25, 0.5);
    viewport.pan(Vec2::new(2.0, -3.0));
    viewport.dolly(-4.0);

    assert_eq!(viewport.size(), Vec2::new(640.0, 480.0));
    assert_approx_eq(viewport.camera().yaw, 0.25);
    assert_approx_eq(viewport.camera().pitch, 0.15);
    assert_eq!(viewport.camera().target, Vec3::new(2.0, -3.0, 0.0));
    assert_approx_eq(viewport.camera().distance, 6.0);
}

#[test]
fn focus_selected_uses_world_bounds_for_bounded_instances() {
    let mut session = open_demo_session();
    let part = add_positioned_part(session.active_scene_mut().unwrap());
    let scene = session.active_scene().unwrap().clone();
    session
        .selection_mut()
        .select_scene_instance(&scene, part)
        .unwrap();

    let result = session.viewport_focus_selected().unwrap();
    let snapshot = session.viewport_snapshot();
    let overlay = snapshot.selection_overlay.expect("selection overlay");

    assert_eq!(result.target.center, Vec3::new(4.0, 2.0, -1.0));
    assert_approx_eq(result.target.radius, 3.0);
    assert_eq!(snapshot.camera.target, Vec3::new(4.0, 2.0, -1.0));
    assert_approx_eq(snapshot.camera.distance, 9.0);
    assert_eq!(overlay.instance_id, part);
    assert!(overlay.bounds.is_some());
}

#[test]
fn focus_selected_falls_back_to_transform_when_bounds_are_not_available() {
    let mut session = open_demo_session();
    let scene = session.active_scene_mut().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let node = scene.add_child(workspace, "Node3D", "Anchor").unwrap();
    scene
        .set_property(
            node,
            "Transform.Position",
            PropertyValue::Vec3(Vec3::new(1.0, 5.0, 9.0)),
        )
        .unwrap();
    let scene = session.active_scene().unwrap().clone();
    session
        .selection_mut()
        .select_scene_instance(&scene, node)
        .unwrap();

    let result = session.viewport_focus_selected().unwrap();

    assert_eq!(result.target.center, Vec3::new(1.0, 5.0, 9.0));
    assert_approx_eq(result.target.radius, 1.0);
    assert!(result.target.bounds.is_none());
}

#[test]
fn focus_selected_reports_clear_error_without_scene_instance_selection() {
    let mut session = open_demo_session();

    let error = session.viewport_focus_selected().unwrap_err();

    assert_eq!(error, ViewportError::NoSceneInstanceSelection);
}

#[test]
fn picking_contract_is_explicit_until_renderer_hit_testing_exists() {
    let viewport = ViewportState::new();

    let inside = viewport.pick(ViewportPickRequest::new(Vec2::new(10.0, 10.0)));
    let outside = viewport.pick(ViewportPickRequest::new(Vec2::new(-1.0, 10.0)));

    assert_eq!(inside.status, ViewportPickStatus::Unsupported);
    assert!(inside.diagnostic.unwrap().contains("renderer-backed"));
    assert_eq!(outside.status, ViewportPickStatus::OutsideViewport);
}

#[test]
fn mcp_snapshot_and_viewport_commands_expose_headless_contract() {
    let mut session = open_demo_session();
    let part = add_positioned_part(session.active_scene_mut().unwrap());
    session.mcp_apply_selection(McpSelectionRequest::SelectSceneInstance { instance_id: part });

    let focus = session.mcp_viewport_focus_selected();
    let pick = session.mcp_viewport_pick(ViewportPickRequest::new(Vec2::new(16.0, 16.0)));
    let snapshot = session.mcp_snapshot();

    assert!(focus.diagnostics.is_empty());
    assert!(focus.viewport.selection_overlay.is_some());
    assert_eq!(pick.pick.status, ViewportPickStatus::Unsupported);
    assert_eq!(
        snapshot.viewport.picking,
        ViewportPickingContract::Placeholder
    );
    assert_eq!(
        snapshot.viewport.selection_overlay.unwrap().instance_id,
        part
    );
}

fn assert_approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "expected {actual} to be within {EPSILON} of {expected}"
    );
}
