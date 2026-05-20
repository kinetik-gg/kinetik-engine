use super::*;

#[test]
fn default_registry_contains_root_services_and_3d_classes_in_order() {
    let registry = InstanceClassRegistry::with_default_scene_classes().unwrap();

    assert_eq!(
        registry.class_names(),
        vec![
            "Game",
            "Workspace",
            "Prefabs",
            "Scripts",
            "UI",
            "Lighting",
            "Audio",
            "Physics",
            "Assets",
            "Packages",
            "Folder",
            "Node3D",
            "Part",
            "Camera3D",
            "Light3D",
        ]
    );
    assert_eq!(registry.descriptors().len(), 15);
}

#[test]
fn default_registry_supports_lookup_by_class_name() {
    let registry = InstanceClassRegistry::with_default_scene_classes().unwrap();

    let game = registry.get(ROOT_CLASS_NAME).unwrap();
    assert_eq!(game.display_name, "Game");
    assert_eq!(
        game.property("Name").unwrap().value_type,
        PropertyType::String
    );

    let workspace = registry.get("Workspace").unwrap();
    assert_eq!(workspace.class_name, "Workspace");
}

#[test]
fn built_in_3d_classes_expose_capabilities_and_shared_properties() {
    let registry = InstanceClassRegistry::with_default_scene_classes().unwrap();

    let folder = registry.get("Folder").unwrap();
    assert!(folder.has_capability(InstanceClassCapability::Container));
    assert!(!folder.has_capability(InstanceClassCapability::Spatial));
    assert!(folder.property("Visible").is_some());

    let part = registry.get("Part").unwrap();
    assert!(part.has_capability(InstanceClassCapability::Spatial));
    assert!(part.has_capability(InstanceClassCapability::Renderable));
    assert_eq!(
        part.property("Transform.Position").unwrap().value_type,
        PropertyType::Vec3
    );
    assert_eq!(
        part.property("Transform.Rotation").unwrap().value_type,
        PropertyType::Quat
    );
    assert_eq!(
        part.property("Transform.Scale").unwrap().value_type,
        PropertyType::Vec3
    );
    assert_eq!(
        part.property("Material.BaseColor").unwrap().value_type,
        PropertyType::Color
    );
    assert_eq!(
        part.property("Material.Metallic").unwrap().value_type,
        PropertyType::F32
    );
    assert_eq!(
        part.property("Material.Roughness").unwrap().value_type,
        PropertyType::F32
    );

    let camera = registry.get("Camera3D").unwrap();
    assert!(camera.has_capability(InstanceClassCapability::Camera));

    let light = registry.get("Light3D").unwrap();
    assert!(light.has_capability(InstanceClassCapability::Light));
}

#[test]
fn registry_rejects_duplicate_classes() {
    let mut registry = InstanceClassRegistry::new();
    registry
        .register(InstanceClassDescriptor::new("Part", "Part").unwrap())
        .unwrap();

    assert_eq!(
        registry
            .register(InstanceClassDescriptor::new("Part", "Part").unwrap())
            .unwrap_err(),
        ClassRegistryError::DuplicateClass {
            class_name: "Part".to_owned()
        }
    );
}

#[test]
fn registry_reports_missing_classes() {
    let registry = InstanceClassRegistry::new();

    assert_eq!(
        registry.get("Missing").unwrap_err(),
        ClassRegistryError::UnknownClass {
            class_name: "Missing".to_owned()
        }
    );
}

#[test]
fn class_descriptors_require_class_names() {
    assert_eq!(
        InstanceClassDescriptor::new(" ", "Empty").unwrap_err(),
        ClassRegistryError::EmptyClassName
    );
}

#[test]
fn class_descriptor_property_lookup_uses_canonical_path() {
    let descriptor = InstanceClassDescriptor::new("TransformNode", "Transform Node")
        .unwrap()
        .with_properties(vec![PropertyDescriptor::new(
            "Transform.Position",
            "Position",
            PropertyType::Vec3,
        )
        .unwrap()]);

    assert!(descriptor.property("Transform.Position").is_some());
    assert!(descriptor.property("transform.position").is_none());
}
