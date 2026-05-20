use kinetik_core::{Aabb, Vec2, Vec3};

use super::*;

fn wall_world() -> StaticCollisionWorld {
    StaticCollisionWorld::from_colliders([StaticCollider::new(
        "Wall",
        Aabb::new(Vec3::new(-0.5, -1.0, -3.0), Vec3::new(0.5, 2.0, -2.0)),
    )])
}

#[test]
fn exposes_crate_name() {
    assert_eq!(crate_name(), "kinetik-physics");
}

#[test]
fn input_frame_stores_actions_in_deterministic_order() {
    let frame = InputFrame::new()
        .with_action(InputAction::MoveRight)
        .with_action(InputAction::MoveForward)
        .with_action(InputAction::MoveRight)
        .with_look_delta(Vec2::new(2.0, -3.0))
        .with_mouse_capture(MouseCapturePolicy::Captured);

    assert_eq!(
        frame.actions(),
        &[InputAction::MoveForward, InputAction::MoveRight]
    );
    assert_eq!(
        frame.movement_intent(),
        Vec2::new(0.707_106_77, 0.707_106_77)
    );
    assert_eq!(frame.look_delta(), Vec2::new(2.0, -3.0));
    assert_eq!(frame.mouse_capture(), MouseCapturePolicy::Captured);
}

#[test]
fn character_controller_moves_relative_to_yaw() {
    let input = InputFrame::from_actions([InputAction::MoveForward]);
    let mut state = CharacterControllerState::new(Vec3::ZERO);
    state.yaw = core::f32::consts::FRAC_PI_2;

    let step = state.step(
        &input,
        0.5,
        CharacterControllerConfig::default(),
        &StaticCollisionWorld::new(),
    );

    assert_approx_eq(step.next.position.x, 2.5);
    assert_approx_eq(step.next.position.z, 0.0);
    assert!(!step.collided);
}

#[test]
fn character_controller_clamps_pitch_and_tracks_look_delta() {
    let input = InputFrame::new().with_look_delta(Vec2::new(10.0, 10.0));
    let config = CharacterControllerConfig::new(5.0, 0.5, 0.5, 0.35, 0.9, 1.0);

    let step = CharacterControllerState::new(Vec3::ZERO).step(
        &input,
        0.0,
        config,
        &StaticCollisionWorld::new(),
    );

    assert_approx_eq(step.next.yaw, 5.0);
    assert_approx_eq(step.next.pitch, 1.0);
}

#[test]
fn character_controller_blocks_static_collision_and_slides_open_axis() {
    let input = InputFrame::from_actions([InputAction::MoveForward, InputAction::MoveRight]);
    let state = CharacterControllerState::new(Vec3::new(-2.0, 0.0, -1.0));

    let step = state.step(
        &input,
        0.5,
        CharacterControllerConfig::default(),
        &wall_world(),
    );

    assert!(step.collided);
    assert!(step.applied_delta.x > 0.0);
    assert_approx_eq(step.applied_delta.z, 0.0);
}

#[test]
fn raycast_static_world_returns_nearest_interaction_hit() {
    let world = StaticCollisionWorld::from_colliders([
        StaticCollider::new(
            "Far",
            Aabb::new(Vec3::new(-0.5, -0.5, -5.0), Vec3::new(0.5, 0.5, -4.0)),
        ),
        StaticCollider::new(
            "Near",
            Aabb::new(Vec3::new(-0.5, -0.5, -3.0), Vec3::new(0.5, 0.5, -2.0)),
        ),
    ]);
    let ray = InteractionRay::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -2.0), 10.0);

    let hit = raycast_static_world(&world, ray).expect("hit");

    assert_eq!(hit.label, "Near");
    assert_approx_eq(hit.distance, 2.0);
    assert_eq!(hit.point, Vec3::new(0.0, 0.0, -2.0));
}

#[test]
fn raycast_static_world_respects_max_distance() {
    let ray = InteractionRay::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), 1.0);

    assert!(raycast_static_world(&wall_world(), ray).is_none());
}

fn assert_approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {actual} to be within tolerance of {expected}"
    );
}
