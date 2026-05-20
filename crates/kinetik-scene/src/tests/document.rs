use super::*;

use std::collections::BTreeMap;

#[test]
fn scene_document_captures_default_scene_shape() {
    let scene = Scene::default_scene().unwrap();
    let document = scene.to_document().unwrap();

    assert_eq!(document.root.guid, InstanceGuid::new(1));
    assert_eq!(document.root.class_name, ROOT_CLASS_NAME);
    assert_eq!(document.root.name, "Game");
    assert_eq!(
        document
            .root
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect::<Vec<_>>(),
        DEFAULT_SERVICE_CLASS_NAMES
    );
}

#[test]
fn scene_document_round_trips_with_deterministic_runtime_ids() {
    let original = Scene::default_scene().unwrap();
    let document = original.to_document().unwrap();
    let restored = Scene::from_document(
        InstanceClassRegistry::with_default_scene_classes().unwrap(),
        document.clone(),
    )
    .unwrap();

    assert_eq!(restored.to_document().unwrap(), document);
    assert_eq!(restored.root_id().unwrap(), InstanceId::new(1));
    assert_eq!(
        restored.get_by_path("/Game/Workspace").unwrap().id,
        InstanceId::new(2)
    );
    assert_eq!(
        restored.get_by_path("/Game/Packages").unwrap().id,
        InstanceId::new(10)
    );
}

#[test]
fn scene_document_properties_are_ordered_by_canonical_path() {
    let mut scene = scene_with_part_class();
    let part = scene.add_root("Part", "Block").unwrap();
    scene
        .set_property(part, "Visible", PropertyValue::Bool(false))
        .unwrap();

    let document = scene.to_document().unwrap();

    assert_eq!(
        document.root.properties.keys().collect::<Vec<_>>(),
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
fn scene_document_rejects_missing_root() {
    let scene = Scene::new();

    assert_eq!(scene.to_document().unwrap_err(), SceneError::MissingRoot);
}

#[test]
fn scene_document_rejects_duplicate_guids() {
    let document = SceneDocument::new(
        SceneInstanceDocument::new(InstanceGuid::new(1), ROOT_CLASS_NAME, "Game").with_children(
            vec![SceneInstanceDocument::new(
                InstanceGuid::new(1),
                "Workspace",
                "Workspace",
            )],
        ),
    );

    assert_eq!(
        Scene::from_document(
            InstanceClassRegistry::with_default_scene_classes().unwrap(),
            document
        )
        .unwrap_err(),
        SceneError::DuplicateInstanceGuid {
            guid: InstanceGuid::new(1)
        }
    );
}

#[test]
fn scene_document_rejects_unknown_classes_and_invalid_properties() {
    let unknown_class = SceneDocument::new(SceneInstanceDocument::new(
        InstanceGuid::new(1),
        "MissingClass",
        "Game",
    ));
    assert_eq!(
        Scene::from_document(
            InstanceClassRegistry::with_default_scene_classes().unwrap(),
            unknown_class
        )
        .unwrap_err(),
        SceneError::UnknownClass {
            class_name: "MissingClass".to_owned()
        }
    );

    let mut unknown_property = BTreeMap::new();
    unknown_property.insert(
        "Missing".to_owned(),
        PropertyValue::String("value".to_owned()),
    );
    let document = SceneDocument::new(
        SceneInstanceDocument::new(InstanceGuid::new(1), ROOT_CLASS_NAME, "Game")
            .with_properties(unknown_property),
    );
    assert_eq!(
        Scene::from_document(
            InstanceClassRegistry::with_default_scene_classes().unwrap(),
            document
        )
        .unwrap_err(),
        SceneError::UnknownProperty {
            class_name: ROOT_CLASS_NAME.to_owned(),
            property_path: "Missing".to_owned()
        }
    );

    let mut mismatched_property = BTreeMap::new();
    mismatched_property.insert("Name".to_owned(), PropertyValue::Bool(true));
    let document = SceneDocument::new(
        SceneInstanceDocument::new(InstanceGuid::new(1), ROOT_CLASS_NAME, "Game")
            .with_properties(mismatched_property),
    );
    assert_eq!(
        Scene::from_document(
            InstanceClassRegistry::with_default_scene_classes().unwrap(),
            document
        )
        .unwrap_err(),
        SceneError::PropertyTypeMismatch {
            property_path: "Name".to_owned(),
            expected: PropertyType::String,
            actual: PropertyType::Bool
        }
    );
}
