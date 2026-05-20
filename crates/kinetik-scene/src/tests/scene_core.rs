use super::*;

#[test]
fn add_and_get_root_instance_by_id_and_guid() {
    let mut scene = Scene::new();
    let id = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let root = scene.get(id).unwrap();

    assert_eq!(scene.root_id(), Some(id));
    assert_eq!(root.guid.raw(), 1);
    assert_eq!(root.class_name, ROOT_CLASS_NAME);
    assert_eq!(root.name, "Game");
    assert_eq!(root.parent, None);
    assert_eq!(scene.get_by_guid(root.guid).unwrap().id, id);
}

#[test]
fn child_ordering_and_paths_are_deterministic() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let lighting = scene.add_child(game, "Lighting", "Lighting").unwrap();
    let audio = scene.add_child(game, "Audio", "Audio").unwrap();

    assert_eq!(scene.children(game).unwrap(), &[workspace, lighting, audio]);
    assert_eq!(scene.path(game).unwrap(), "/Game");
    assert_eq!(scene.path(workspace).unwrap(), "/Game/Workspace");
    assert_eq!(scene.path(lighting).unwrap(), "/Game/Lighting");
    assert_eq!(scene.get_by_path("/Game/Audio").unwrap().id, audio);
}

#[test]
fn nested_paths_resolve_through_ordered_children() {
    let mut scene = Scene::new();

    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let folder = scene.add_child(workspace, "Folder", "Enemies").unwrap();

    assert_eq!(scene.path(folder).unwrap(), "/Game/Workspace/Enemies");
    assert_eq!(
        scene.get_by_path("/Game/Workspace/Enemies").unwrap().id,
        folder
    );
}

#[test]
fn scene_rejects_duplicate_roots() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();

    assert_eq!(
        scene.add_root(ROOT_CLASS_NAME, "OtherGame").unwrap_err(),
        SceneError::DuplicateRoot { root_id: game }
    );
}

#[test]
fn scene_reports_invalid_handles_and_paths() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();

    assert_eq!(
        scene.get(InstanceId::new(99)).unwrap_err(),
        SceneError::InvalidInstanceId {
            id: InstanceId::new(99)
        }
    );
    assert_eq!(
        scene.get_by_guid(InstanceGuid::new(99)).unwrap_err(),
        SceneError::InvalidInstanceGuid {
            guid: InstanceGuid::new(99)
        }
    );
    assert_eq!(
        scene.get_by_path("Game").unwrap_err(),
        SceneError::InvalidPath {
            path: "Game".to_owned()
        }
    );
    assert_eq!(
        scene.get_by_path("/Game/Missing").unwrap_err(),
        SceneError::InvalidPath {
            path: "/Game/Missing".to_owned()
        }
    );
    assert_eq!(scene.path(game).unwrap(), "/Game");
}

#[test]
fn scene_validates_classes_and_names() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();

    assert_eq!(
        scene.add_child(game, "MissingClass", "Thing").unwrap_err(),
        SceneError::UnknownClass {
            class_name: "MissingClass".to_owned()
        }
    );
    assert_eq!(
        scene.add_child(game, "Workspace", "Bad/Name").unwrap_err(),
        SceneError::InvalidInstanceName {
            name: "Bad/Name".to_owned()
        }
    );
}

#[test]
fn instance_properties_start_with_descriptor_defaults() {
    let mut scene = scene_with_part_class();
    let part = scene.add_root("Part", "Block").unwrap();

    assert_eq!(
        scene.get_property(part, "Name").unwrap(),
        &PropertyValue::String("Block".to_owned())
    );
    assert_eq!(
        scene.get_property(part, "Visible").unwrap(),
        &PropertyValue::Bool(true)
    );
    assert_eq!(
        scene.get_property(part, "Transform.Position").unwrap(),
        &PropertyValue::Vec3(kinetik_core::Vec3::ZERO)
    );
    assert_eq!(
        scene.get_property(part, "Transform.Rotation").unwrap(),
        &PropertyValue::Quat(kinetik_core::Quat::IDENTITY)
    );
    assert_eq!(
        scene.get_property(part, "Transform.Scale").unwrap(),
        &PropertyValue::Vec3(kinetik_core::Vec3::ONE)
    );
    assert_eq!(
        scene.get_property(part, "Material.BaseColor").unwrap(),
        &PropertyValue::Color(kinetik_core::Color::rgb(0.78, 0.82, 0.88))
    );
    assert_eq!(
        scene.get_property(part, "Material.Metallic").unwrap(),
        &PropertyValue::F32(0.0)
    );
    assert_eq!(
        scene.get_property(part, "Material.Roughness").unwrap(),
        &PropertyValue::F32(0.65)
    );
    assert_eq!(
        scene.properties(part).unwrap().keys().collect::<Vec<_>>(),
        vec![
            "Material.BaseColor",
            "Material.Metallic",
            "Material.Roughness",
            "Name",
            "Transform.Position",
            "Transform.Rotation",
            "Transform.Scale",
            "Visible"
        ]
    );
}

#[test]
fn set_property_validates_and_stores_values() {
    let mut scene = scene_with_part_class();
    let part = scene.add_root("Part", "Block").unwrap();

    scene
        .set_property(part, "Visible", PropertyValue::Bool(false))
        .unwrap();
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
            "Transform.Scale",
            PropertyValue::Vec3(kinetik_core::Vec3::new(2.0, 2.0, 2.0)),
        )
        .unwrap();

    assert_eq!(
        scene.get_property(part, "Visible").unwrap(),
        &PropertyValue::Bool(false)
    );
    assert_eq!(
        scene.get_property(part, "Transform.Position").unwrap(),
        &PropertyValue::Vec3(kinetik_core::Vec3::new(1.0, 2.0, 3.0))
    );
    assert_eq!(
        scene.get_property(part, "Transform.Scale").unwrap(),
        &PropertyValue::Vec3(kinetik_core::Vec3::new(2.0, 2.0, 2.0))
    );
}

#[test]
fn name_property_updates_instance_name_and_path() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();

    scene
        .set_property(
            workspace,
            "Name",
            PropertyValue::String("WorldRoot".to_owned()),
        )
        .unwrap();

    assert_eq!(scene.get(workspace).unwrap().name, "WorldRoot");
    assert_eq!(scene.path(workspace).unwrap(), "/Game/WorldRoot");
    assert_eq!(
        scene.get_property(workspace, "Name").unwrap(),
        &PropertyValue::String("WorldRoot".to_owned())
    );
}

#[test]
fn property_storage_rejects_unknown_and_noncanonical_paths() {
    let mut scene = scene_with_part_class();
    let part = scene.add_root("Part", "Block").unwrap();

    assert_eq!(
        scene.get_property(part, "visible").unwrap_err(),
        SceneError::UnknownProperty {
            class_name: "Part".to_owned(),
            property_path: "visible".to_owned()
        }
    );
    assert_eq!(
        scene
            .set_property(
                part,
                "Transform.position",
                PropertyValue::Vec3(kinetik_core::Vec3::ZERO)
            )
            .unwrap_err(),
        SceneError::UnknownProperty {
            class_name: "Part".to_owned(),
            property_path: "Transform.position".to_owned()
        }
    );
}

#[test]
fn property_storage_rejects_type_mismatches() {
    let mut scene = scene_with_part_class();
    let part = scene.add_root("Part", "Block").unwrap();

    assert_eq!(
        scene
            .set_property(part, "Visible", PropertyValue::String("yes".to_owned()))
            .unwrap_err(),
        SceneError::PropertyTypeMismatch {
            property_path: "Visible".to_owned(),
            expected: PropertyType::Bool,
            actual: PropertyType::String
        }
    );
}
