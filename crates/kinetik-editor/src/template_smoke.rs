use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use kinetik_core::{Aabb, Vec2, Vec3};
use kinetik_physics::{
    raycast_static_world, CharacterControllerConfig, CharacterControllerState, InputAction,
    InputFrame, InteractionRay, StaticCollider, StaticCollisionWorld,
};
use kinetik_render::{extract_render_scene, render_smoke_image, StandardMaterial};
use kinetik_scene::{InstanceClassRegistry, SceneDocument};

use crate::{EditorModeState, EditorSession};

#[test]
fn primitive_showcase_template_loads_saves_and_runs_headless_smoke() {
    let template_root = primitive_showcase_root();
    let mut session = load_template(&template_root);

    let scene = session.active_scene().expect("active scene");
    assert!(scene.get_by_path("/Game/Workspace/Camera").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/KeyLight").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/BaseBlock").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/TallBlock").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/OffsetBlock").is_ok());

    let extraction = extract_render_scene(scene);
    assert!(extraction.camera.is_some());
    assert_eq!(extraction.lights.len(), 1);
    assert_eq!(extraction.primitives.len(), 3);
    assert!(extraction.diagnostics.is_empty());

    let smoke = render_smoke_image(&extraction, 128, 72);
    assert!(smoke.has_non_background_pixels());

    let saved_root = temp_project_root("primitive-showcase-save");
    session.save_project_to(&saved_root).expect("save template");
    assert_template_file_matches(&template_root, &saved_root, "Kinetik.toml");
    assert_template_file_matches(&template_root, &saved_root, "project/assets.knmanifest");
    assert_template_file_matches(&template_root, &saved_root, "scenes/main.knscene");
    std::fs::remove_dir_all(saved_root).expect("cleanup saved template");
}

#[test]
fn pbr_material_demo_template_loads_saves_and_runs_headless_smoke() {
    let template_root = template_root("pbr-material-demo");
    let mut session = load_template(&template_root);

    let scene = session.active_scene().expect("active scene");
    assert!(scene.get_by_path("/Game/Workspace/Camera").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/KeyLight").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/DielectricRough").is_ok());
    assert!(scene
        .get_by_path("/Game/Workspace/DielectricSmooth")
        .is_ok());
    assert!(scene.get_by_path("/Game/Workspace/MetalRough").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/MetalSmooth").is_ok());

    let extraction = extract_render_scene(scene);
    assert!(extraction.camera.is_some());
    assert_eq!(extraction.lights.len(), 1);
    assert_eq!(extraction.primitives.len(), 4);
    assert!(extraction.diagnostics.is_empty());
    assert_eq!(
        extraction
            .primitives
            .iter()
            .map(|primitive| (primitive.material.metallic, primitive.material.roughness))
            .collect::<Vec<_>>(),
        vec![(0.0, 0.9), (0.0, 0.15), (1.0, 0.75), (1.0, 0.1)]
    );
    assert!(extraction
        .primitives
        .iter()
        .any(|primitive| primitive.material != StandardMaterial::FALLBACK));

    let smoke = render_smoke_image(&extraction, 128, 72);
    assert!(smoke.has_non_background_pixels());

    let saved_root = temp_project_root("pbr-material-demo-save");
    session.save_project_to(&saved_root).expect("save template");
    assert_template_file_matches(&template_root, &saved_root, "Kinetik.toml");
    assert_template_file_matches(&template_root, &saved_root, "project/assets.knmanifest");
    assert_template_file_matches(&template_root, &saved_root, "scenes/main.knscene");
    std::fs::remove_dir_all(saved_root).expect("cleanup saved template");
}

#[test]
fn basic_fps_template_loads_saves_and_runs_headless_objective_smoke() {
    let template_root = template_root("basic-fps");
    let mut session = load_template(&template_root);

    let scene = session.active_scene().expect("active scene");
    assert!(scene.get_by_path("/Game/Workspace/PlayerStart").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/Camera").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/Floor").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/Wall").is_ok());
    assert!(scene.get_by_path("/Game/Workspace/Objective").is_ok());

    let extraction = extract_render_scene(scene);
    assert!(extraction.camera.is_some());
    assert_eq!(extraction.lights.len(), 1);
    assert_eq!(extraction.primitives.len(), 3);
    assert!(render_smoke_image(&extraction, 128, 72).has_non_background_pixels());

    let outcome = run_basic_fps_headless_objective();
    assert!(outcome.completed);
    assert!(outcome.collided);
    assert_eq!(outcome.restart.position, Vec3::new(0.0, 1.0, 2.0));

    session.start_play_mode().expect("start play mode");
    session.step_play_mode(1.0 / 60.0).expect("step play");
    session.stop_play_mode();
    assert!(session.dirty_state().is_clean());

    let saved_root = temp_project_root("basic-fps-save");
    session.save_project_to(&saved_root).expect("save template");
    assert_template_file_matches(&template_root, &saved_root, "Kinetik.toml");
    assert_template_file_matches(&template_root, &saved_root, "project/assets.knmanifest");
    assert_template_scene_document_matches(&template_root, &saved_root, "scenes/main.knscene");
    std::fs::remove_dir_all(saved_root).expect("cleanup saved template");
}

#[test]
fn primitive_showcase_play_mode_does_not_persist_runtime_state() {
    let template_root = primitive_showcase_root();
    let mut session = load_template(&template_root);
    let edit_scene_before = session.active_scene().unwrap().to_document().unwrap();

    session.start_play_mode().expect("start play mode");
    assert_eq!(session.mode(), EditorModeState::Play);
    let step = session.step_play_mode(1.0 / 60.0).expect("step play mode");
    assert_eq!(step.frame_index, 1);
    assert!(session
        .diagnostics_panel()
        .items()
        .iter()
        .any(|item| item.code == "KT_RUNTIME_PLAY_STATE"));

    session.stop_play_mode();

    assert_eq!(session.mode(), EditorModeState::Edit);
    assert_eq!(
        session.active_scene().unwrap().to_document().unwrap(),
        edit_scene_before
    );
    assert!(session.dirty_state().is_clean());
}

fn load_template(template_root: &Path) -> EditorSession {
    let mut session = EditorSession::new();
    session
        .reload_project_from(template_root, default_scene_registry())
        .expect("load primitive showcase template");
    session
}

fn primitive_showcase_root() -> PathBuf {
    template_root("primitive-showcase")
}

fn template_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../templates")
        .join(name)
        .canonicalize()
        .expect("template path")
}

fn default_scene_registry() -> InstanceClassRegistry {
    InstanceClassRegistry::with_default_scene_classes().expect("built-in scene classes")
}

fn assert_template_file_matches(template_root: &Path, saved_root: &Path, path: &str) {
    let template = std::fs::read_to_string(template_root.join(path)).expect("read template file");
    let saved = std::fs::read_to_string(saved_root.join(path)).expect("read saved file");
    assert_eq!(saved, template, "{path} should round-trip byte-for-byte");
}

fn assert_template_scene_document_matches(template_root: &Path, saved_root: &Path, path: &str) {
    let template = std::fs::read_to_string(template_root.join(path)).expect("read template scene");
    let saved = std::fs::read_to_string(saved_root.join(path)).expect("read saved scene");
    assert_eq!(
        SceneDocument::from_ron_str(&saved).expect("parse saved scene"),
        SceneDocument::from_ron_str(&template).expect("parse template scene"),
        "{path} should round-trip semantically"
    );
}

struct BasicFpsOutcome {
    completed: bool,
    collided: bool,
    restart: CharacterControllerState,
}

fn run_basic_fps_headless_objective() -> BasicFpsOutcome {
    let wall = StaticCollider::new(
        "Wall",
        Aabb::new(Vec3::new(-1.5, -0.5, -3.0), Vec3::new(1.5, 1.5, -2.6)),
    );
    let objective = StaticCollider::new(
        "Objective",
        Aabb::new(
            Vec3::new(1.625, -0.175, 1.625),
            Vec3::new(2.375, 1.5, 2.375),
        ),
    );
    let movement_world = StaticCollisionWorld::from_colliders([wall.clone()]);
    let interaction_world = StaticCollisionWorld::from_colliders([wall, objective]);
    let config = CharacterControllerConfig::default();
    let start = CharacterControllerState::new(Vec3::new(0.0, 1.0, 2.0));
    let blocked = start.step(
        &InputFrame::from_actions([InputAction::MoveForward]),
        1.0,
        config,
        &movement_world,
    );
    let moved = blocked.next.step(
        &InputFrame::from_actions([InputAction::MoveRight]).with_look_delta(Vec2::new(0.0, 10.0)),
        0.4,
        config,
        &movement_world,
    );
    let ray = InteractionRay::new(moved.next.position, moved.next.right(), 3.0);
    let completed =
        raycast_static_world(&interaction_world, ray).is_some_and(|hit| hit.label == "Objective");

    BasicFpsOutcome {
        completed,
        collided: blocked.collided,
        restart: start,
    }
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
