use super::*;
use kinetik_scene::ROOT_CLASS_NAME;

#[test]
fn runtime_world_clone_preserves_edit_guid_mapping() {
    let scene = kinetik_scene::Scene::default_scene().unwrap();
    let document = scene.to_document().unwrap();
    let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();

    assert_eq!(world.id(), RuntimeWorldId::new(1));
    assert_eq!(world.root_id(), Some(RuntimeInstanceId::new(1)));
    assert_eq!(
        world.runtime_id_for_edit_guid(document.root.guid),
        Some(RuntimeInstanceId::new(1))
    );
    assert_eq!(
        world.get(RuntimeInstanceId::new(1)).unwrap().class_name,
        ROOT_CLASS_NAME
    );
    assert_eq!(
        world.get(RuntimeInstanceId::new(2)).unwrap().parent,
        Some(RuntimeInstanceId::new(1))
    );
}

#[test]
fn runtime_world_clone_uses_deterministic_parent_before_child_order() {
    let scene = kinetik_scene::Scene::default_scene().unwrap();
    let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(7), &scene).unwrap();
    let names: Vec<&str> = world
        .instances()
        .iter()
        .map(|instance| instance.name.as_str())
        .collect();

    assert_eq!(
        names,
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
        ]
    );
    assert_eq!(
        world.get(RuntimeInstanceId::new(1)).unwrap().children,
        (2..=10).map(RuntimeInstanceId::new).collect::<Vec<_>>()
    );
}

#[test]
fn runtime_world_clone_does_not_require_edit_instance_ids() {
    let mut scene = kinetik_scene::Scene::new();
    let root = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let node = scene.add_child(root, "Node3D", "Node").unwrap();
    let edit_root_raw = root.raw();
    let edit_node_raw = node.raw();

    let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(3), &scene).unwrap();

    assert_eq!(world.root_id().unwrap().raw(), 1);
    assert_eq!(world.get(RuntimeInstanceId::new(2)).unwrap().name, "Node");
    assert_eq!(edit_root_raw, 1);
    assert_eq!(edit_node_raw, 2);
    assert_ne!(
        core::any::type_name::<RuntimeInstanceId>(),
        core::any::type_name::<kinetik_core::InstanceId>()
    );
}

#[test]
fn runtime_world_clone_requires_edit_scene_root() {
    let scene = kinetik_scene::Scene::new();

    assert_eq!(
        RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap_err(),
        SceneError::MissingRoot
    );
}

#[test]
fn runtime_spawn_creates_runtime_only_child_identity() {
    let scene = kinetik_scene::Scene::default_scene().unwrap();
    let mut world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();
    let parent = RuntimeInstanceId::new(2);

    let spawned = world
        .spawn_runtime_child(parent, "Part", "RuntimeBlock")
        .unwrap();

    let spawned_record = world.get(spawned).unwrap();
    assert_eq!(spawned, RuntimeInstanceId::new(11));
    assert_eq!(spawned_record.edit_guid, None);
    assert_eq!(spawned_record.parent, Some(parent));
    assert_eq!(spawned_record.class_name, "Part");
    assert_eq!(spawned_record.name, "RuntimeBlock");
    assert_eq!(
        world.get(parent).unwrap().children.last().copied(),
        Some(spawned)
    );
}

#[test]
fn runtime_spawn_rejects_missing_parent_and_invalid_names() {
    let scene = kinetik_scene::Scene::default_scene().unwrap();
    let mut world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();

    assert_eq!(
        world
            .spawn_runtime_child(RuntimeInstanceId::new(99), "Part", "Block")
            .unwrap_err(),
        RuntimeWorldError::MissingParent {
            parent: RuntimeInstanceId::new(99)
        }
    );
    assert_eq!(
        world
            .spawn_runtime_child(RuntimeInstanceId::new(1), "  ", "Block")
            .unwrap_err(),
        RuntimeWorldError::EmptyClassName
    );
    assert_eq!(
        world
            .spawn_runtime_child(RuntimeInstanceId::new(1), "Part", "Bad/Name")
            .unwrap_err(),
        RuntimeWorldError::InvalidInstanceName {
            name: "Bad/Name".to_owned()
        }
    );
}

#[test]
fn runtime_despawn_removes_subtree_and_parent_links() {
    let scene = kinetik_scene::Scene::default_scene().unwrap();
    let mut world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();
    let parent = RuntimeInstanceId::new(2);
    let child = world
        .spawn_runtime_child(parent, "Node3D", "RuntimeParent")
        .unwrap();
    let grandchild = world
        .spawn_runtime_child(child, "Part", "RuntimeChild")
        .unwrap();

    let removed = world.despawn_runtime_subtree(child).unwrap();

    assert_eq!(removed, vec![child, grandchild]);
    assert!(world.get(child).is_none());
    assert!(world.get(grandchild).is_none());
    assert!(!world.get(parent).unwrap().children.contains(&child));
}

#[test]
fn runtime_despawn_rejects_missing_instance_and_root() {
    let scene = kinetik_scene::Scene::default_scene().unwrap();
    let mut world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();

    assert_eq!(
        world
            .despawn_runtime_subtree(RuntimeInstanceId::new(99))
            .unwrap_err(),
        RuntimeWorldError::InvalidInstance {
            id: RuntimeInstanceId::new(99)
        }
    );
    assert_eq!(
        world
            .despawn_runtime_subtree(world.root_id().unwrap())
            .unwrap_err(),
        RuntimeWorldError::CannotDespawnRoot {
            root: RuntimeInstanceId::new(1)
        }
    );
}

#[test]
fn runtime_mutations_do_not_mutate_source_edit_scene() {
    let mut scene = kinetik_scene::Scene::new();
    let root = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let edit_child = scene.add_child(root, "Node3D", "SavedNode").unwrap();
    let edit_child_guid = scene.get(edit_child).unwrap().guid;
    let edit_document_before = scene.to_document().unwrap();
    let mut world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();
    let runtime_child = world.runtime_id_for_edit_guid(edit_child_guid).unwrap();

    let spawned = world
        .spawn_runtime_child(runtime_child, "Part", "RuntimeBlock")
        .unwrap();
    assert_eq!(world.get(spawned).unwrap().edit_guid, None);
    assert_eq!(
        world.despawn_runtime_subtree(runtime_child).unwrap(),
        vec![runtime_child, spawned]
    );

    assert_eq!(scene.to_document().unwrap(), edit_document_before);
    assert!(world.runtime_id_for_edit_guid(edit_child_guid).is_none());
}
