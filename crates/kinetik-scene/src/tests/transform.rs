use super::*;

#[test]
fn local_transform_reads_reflected_transform_properties() {
    let mut scene = Scene::new();
    let part = scene.add_root("Part", "Block").unwrap();

    scene
        .set_property(
            part,
            "Transform.Position",
            PropertyValue::Vec3(kinetik_core::Vec3::new(1.0, 2.0, 3.0)),
        )
        .unwrap();
    scene
        .set_property(
            part,
            "Transform.Rotation",
            PropertyValue::Quat(kinetik_core::Quat::new(0.0, 0.0, 1.0, 0.0)),
        )
        .unwrap();
    scene
        .set_property(
            part,
            "Transform.Scale",
            PropertyValue::Vec3(kinetik_core::Vec3::new(2.0, 3.0, 4.0)),
        )
        .unwrap();

    assert_eq!(
        scene.local_transform(part).unwrap(),
        kinetik_core::Transform::new(
            kinetik_core::Vec3::new(1.0, 2.0, 3.0),
            kinetik_core::Quat::new(0.0, 0.0, 1.0, 0.0),
            kinetik_core::Vec3::new(2.0, 3.0, 4.0)
        )
    );
}

#[test]
fn world_transform_composes_spatial_ancestors_deterministically() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let parent = scene.add_child(workspace, "Node3D", "Parent").unwrap();
    let child = scene.add_child(parent, "Part", "Child").unwrap();

    scene
        .set_property(
            parent,
            "Transform.Position",
            PropertyValue::Vec3(kinetik_core::Vec3::new(10.0, 0.0, 0.0)),
        )
        .unwrap();
    scene
        .set_property(
            parent,
            "Transform.Rotation",
            PropertyValue::Quat(kinetik_core::Quat::new(0.0, 0.0, 1.0, 0.0)),
        )
        .unwrap();
    scene
        .set_property(
            parent,
            "Transform.Scale",
            PropertyValue::Vec3(kinetik_core::Vec3::new(2.0, 1.0, 1.0)),
        )
        .unwrap();
    scene
        .set_property(
            child,
            "Transform.Position",
            PropertyValue::Vec3(kinetik_core::Vec3::X),
        )
        .unwrap();
    scene
        .set_property(
            child,
            "Transform.Scale",
            PropertyValue::Vec3(kinetik_core::Vec3::new(3.0, 4.0, 5.0)),
        )
        .unwrap();

    assert_eq!(
        scene.world_transform(child).unwrap(),
        kinetik_core::Transform::new(
            kinetik_core::Vec3::new(8.0, 0.0, 0.0),
            kinetik_core::Quat::new(0.0, 0.0, 1.0, 0.0),
            kinetik_core::Vec3::new(6.0, 4.0, 5.0)
        )
    );
}

#[test]
fn transform_queries_reject_non_spatial_targets() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();

    assert_eq!(
        scene.local_transform(game).unwrap_err(),
        SceneError::NonSpatialInstance {
            id: game,
            class_name: ROOT_CLASS_NAME.to_owned()
        }
    );
    assert_eq!(
        scene.world_transform(game).unwrap_err(),
        SceneError::NonSpatialInstance {
            id: game,
            class_name: ROOT_CLASS_NAME.to_owned()
        }
    );
}

#[test]
fn transform_revision_advances_for_hierarchy_and_transform_edits() {
    let mut scene = Scene::new();
    assert_eq!(scene.transform_revision(), 0);

    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    assert_eq!(scene.transform_revision(), 1);
    let part = scene.add_child(game, "Part", "Block").unwrap();
    assert_eq!(scene.transform_revision(), 2);

    scene
        .set_property(part, "Visible", PropertyValue::Bool(false))
        .unwrap();
    assert_eq!(scene.transform_revision(), 2);
    scene
        .set_property(
            part,
            "Transform.Position",
            PropertyValue::Vec3(kinetik_core::Vec3::X),
        )
        .unwrap();
    assert_eq!(scene.transform_revision(), 3);
    scene
        .set_property(
            part,
            "Transform.Rotation",
            PropertyValue::Quat(kinetik_core::Quat::new(0.0, 0.0, 1.0, 0.0)),
        )
        .unwrap();
    assert_eq!(scene.transform_revision(), 4);
}

#[test]
fn transform_revision_advances_only_for_successful_transform_changes() {
    let mut scene = Scene::new();
    let part = scene.add_root("Part", "Block").unwrap();
    let revision = scene.transform_revision();

    assert_eq!(
        scene
            .set_property(part, "Transform.Position", PropertyValue::Bool(true))
            .unwrap_err(),
        SceneError::PropertyTypeMismatch {
            property_path: "Transform.Position".to_owned(),
            expected: PropertyType::Vec3,
            actual: PropertyType::Bool
        }
    );
    assert_eq!(scene.transform_revision(), revision);

    scene
        .set_property(part, "Name", PropertyValue::String("Renamed".to_owned()))
        .unwrap();
    assert_eq!(scene.transform_revision(), revision);
}

#[test]
fn transform_revision_tracks_reparent_and_delete_mutations() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let node = scene.add_child(game, "Node3D", "Node").unwrap();
    let part = scene.add_child(workspace, "Part", "Block").unwrap();
    let revision = scene.transform_revision();

    let mut no_op_reparent = SceneMutationQueue::new();
    no_op_reparent.reparent(part, workspace);
    scene.apply_mutations(no_op_reparent).unwrap();
    assert_eq!(scene.transform_revision(), revision);

    let mut reparent = SceneMutationQueue::new();
    reparent.reparent(part, node);
    scene.apply_mutations(reparent).unwrap();
    assert_eq!(scene.transform_revision(), revision + 1);

    let mut delete = SceneMutationQueue::new();
    delete.delete(part);
    scene.apply_mutations(delete).unwrap();
    assert_eq!(scene.transform_revision(), revision + 2);
}

#[test]
fn local_bounds_follow_approved_part_unit_cube_contract() {
    let mut scene = Scene::new();
    let part = scene.add_root("Part", "Block").unwrap();

    assert_eq!(scene.local_bounds(part).unwrap(), PART_LOCAL_BOUNDS);
    assert_eq!(PART_LOCAL_BOUNDS.size(), kinetik_core::Vec3::ONE);
    assert_eq!(PART_LOCAL_BOUNDS.center(), kinetik_core::Vec3::ZERO);
}

#[test]
fn world_bounds_transform_all_local_part_corners() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let parent = scene.add_child(game, "Node3D", "Parent").unwrap();
    let part = scene.add_child(parent, "Part", "Block").unwrap();

    scene
        .set_property(
            parent,
            "Transform.Position",
            PropertyValue::Vec3(kinetik_core::Vec3::new(10.0, 0.0, 0.0)),
        )
        .unwrap();
    scene
        .set_property(
            parent,
            "Transform.Rotation",
            PropertyValue::Quat(kinetik_core::Quat::new(0.0, 0.0, 1.0, 0.0)),
        )
        .unwrap();
    scene
        .set_property(
            parent,
            "Transform.Scale",
            PropertyValue::Vec3(kinetik_core::Vec3::new(2.0, 1.0, 1.0)),
        )
        .unwrap();
    scene
        .set_property(
            part,
            "Transform.Position",
            PropertyValue::Vec3(kinetik_core::Vec3::X),
        )
        .unwrap();
    scene
        .set_property(
            part,
            "Transform.Scale",
            PropertyValue::Vec3(kinetik_core::Vec3::new(3.0, 4.0, 5.0)),
        )
        .unwrap();

    assert_eq!(
        scene.world_bounds(part).unwrap(),
        kinetik_core::Aabb::new(
            kinetik_core::Vec3::new(5.0, -2.0, -2.5),
            kinetik_core::Vec3::new(11.0, 2.0, 2.5)
        )
    );
}

#[test]
fn bounds_queries_report_no_bounds_for_non_bounded_classes() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let folder = scene.add_child(game, "Folder", "Folder").unwrap();
    let node = scene.add_child(folder, "Node3D", "Node").unwrap();
    let camera = scene.add_child(node, "Camera3D", "Camera").unwrap();
    let light = scene.add_child(node, "Light3D", "Light").unwrap();

    for (id, class_name) in [
        (game, ROOT_CLASS_NAME),
        (folder, "Folder"),
        (node, "Node3D"),
        (camera, "Camera3D"),
        (light, "Light3D"),
    ] {
        assert_eq!(
            scene.local_bounds(id).unwrap_err(),
            SceneError::NoBounds {
                id,
                class_name: class_name.to_owned()
            }
        );
        assert_eq!(
            scene.world_bounds(id).unwrap_err(),
            SceneError::NoBounds {
                id,
                class_name: class_name.to_owned()
            }
        );
    }
}
