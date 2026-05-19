use super::*;

#[test]
fn default_scene_has_exact_adr_0002_hierarchy() {
    let scene = Scene::default_scene().unwrap();
    let root_id = scene.root_id().unwrap();
    let root = scene.get(root_id).unwrap();

    assert_eq!(root.name, "Game");
    assert_eq!(root.class_name, ROOT_CLASS_NAME);
    assert_eq!(scene.path(root_id).unwrap(), "/Game");

    let service_names: Vec<&str> = scene
        .children(root_id)
        .unwrap()
        .iter()
        .map(|id| scene.get(*id).unwrap().name.as_str())
        .collect();
    assert_eq!(service_names, DEFAULT_SERVICE_CLASS_NAMES);
}

#[test]
fn default_scene_services_are_visible_by_path() {
    let scene = Scene::default_scene().unwrap();

    for class_name in DEFAULT_SERVICE_CLASS_NAMES {
        let path = format!("/Game/{class_name}");
        let service = scene.get_by_path(&path).unwrap();
        assert_eq!(service.name, class_name);
        assert_eq!(service.class_name, class_name);
    }
}

#[test]
fn default_scene_ids_are_deterministic() {
    let first = Scene::default_scene().unwrap();
    let second = Scene::default_scene().unwrap();

    assert_eq!(first.root_id().unwrap().raw(), 1);
    assert_eq!(second.root_id().unwrap().raw(), 1);

    for (index, class_name) in DEFAULT_SERVICE_CLASS_NAMES.iter().enumerate() {
        let expected_raw = index as u64 + 2;
        let path = format!("/Game/{class_name}");
        let first_service = first.get_by_path(&path).unwrap();
        let second_service = second.get_by_path(&path).unwrap();

        assert_eq!(first_service.id.raw(), expected_raw);
        assert_eq!(first_service.guid.raw(), expected_raw);
        assert_eq!(second_service.id.raw(), expected_raw);
        assert_eq!(second_service.guid.raw(), expected_raw);
    }
}
