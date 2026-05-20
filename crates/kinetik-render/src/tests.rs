use kinetik_core::{Color, DiagnosticBlockingScope, Vec3};
use kinetik_reflect::PropertyValue;
use kinetik_scene::Scene;

use super::*;

fn primitive_scene() -> Scene {
    let mut scene = Scene::default_scene().expect("default scene");
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    scene
        .add_child(workspace, "Camera3D", "Camera")
        .expect("camera");
    scene.add_child(workspace, "Light3D", "Key").expect("light");
    let part = scene.add_child(workspace, "Part", "Block").expect("part");
    scene
        .set_property(
            part,
            "Transform.Position",
            PropertyValue::Vec3(Vec3::new(1.0, 2.0, 0.0)),
        )
        .unwrap();
    scene
        .set_property(
            part,
            "Transform.Scale",
            PropertyValue::Vec3(Vec3::new(2.0, 1.0, 1.0)),
        )
        .unwrap();
    scene
}

#[test]
fn exposes_crate_name() {
    assert_eq!(crate_name(), "kinetik-render");
}

#[test]
fn standard_material_clamps_pbr_factors_and_has_safe_fallback() {
    let material = StandardMaterial::new(Color::rgb(0.2, 0.4, 0.6), 2.0, -1.0);

    assert_eq!(material.base_color, Color::rgb(0.2, 0.4, 0.6));
    assert_approx_eq(material.metallic, 1.0);
    assert_approx_eq(material.roughness, 0.0);
    assert_eq!(StandardMaterial::default(), StandardMaterial::FALLBACK);
}

#[test]
fn primitive_mesh_contract_exposes_builtin_cube() {
    let cube = PrimitiveMesh::cube();

    assert_eq!(cube.kind, PrimitiveMeshKind::Cube);
    assert_eq!(cube.resource_path, "builtin://mesh/cube");
    assert_eq!(cube.min, Vec3::new(-0.5, -0.5, -0.5));
    assert_eq!(cube.max, Vec3::new(0.5, 0.5, 0.5));
}

#[test]
fn extraction_collects_camera_light_and_visible_part_with_fallback_material() {
    let extraction = extract_render_scene(&primitive_scene());

    assert!(extraction.camera.is_some());
    assert_eq!(extraction.lights.len(), 1);
    assert_eq!(extraction.primitives.len(), 1);
    assert_eq!(extraction.primitives[0].mesh.kind, PrimitiveMeshKind::Cube);
    assert_eq!(
        extraction.primitives[0].material,
        StandardMaterial::FALLBACK
    );
    assert!(extraction
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == MISSING_MATERIAL_CODE));
}

#[test]
fn extraction_reports_missing_render_scene_requirements() {
    let scene = Scene::default_scene().expect("default scene");
    let extraction = extract_render_scene(&scene);

    let codes = extraction
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&MISSING_CAMERA_CODE.as_str()));
    assert!(codes.contains(&MISSING_LIGHT_CODE.as_str()));
    assert!(codes.contains(&MISSING_MESH_CODE.as_str()));
    assert!(extraction
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.blocking == Some(DiagnosticBlockingScope::Play)));
}

#[test]
fn invisible_parts_do_not_extract_as_render_primitives() {
    let mut scene = primitive_scene();
    let part = scene.get_by_path("/Game/Workspace/Block").unwrap().id;
    scene
        .set_property(part, "Visible", PropertyValue::Bool(false))
        .unwrap();

    let extraction = extract_render_scene(&scene);

    assert!(extraction.primitives.is_empty());
    assert!(extraction
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == MISSING_MESH_CODE));
}

#[test]
fn smoke_image_is_deterministic_and_nonblank_for_primitive_scene() {
    let extraction = extract_render_scene(&primitive_scene());

    let first = render_smoke_image(&extraction, 64, 48);
    let second = render_smoke_image(&extraction, 64, 48);

    assert_eq!(first, second);
    assert_eq!(first.width(), 64);
    assert_eq!(first.height(), 48);
    assert!(first.has_non_background_pixels());
}

fn assert_approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {actual} to be within tolerance of {expected}"
    );
}
