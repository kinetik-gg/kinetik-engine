use super::*;

#[test]
fn mutation_queue_applies_valid_batch_in_order() {
    let mut scene = Scene::default_scene().unwrap();
    let game = scene.root_id().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let audio = scene.get_by_path("/Game/Audio").unwrap().id;

    let mut queue = SceneMutationQueue::new();
    queue.rename(workspace, "World");
    queue.reparent(audio, workspace);
    queue.create_child(workspace, "Workspace", "Zone");

    let results = scene.apply_mutations(queue).unwrap();

    assert_eq!(
        results,
        vec![
            SceneMutationResult::Renamed { id: workspace },
            SceneMutationResult::Reparented {
                id: audio,
                old_parent: Some(game),
                new_parent: workspace
            },
            SceneMutationResult::Created {
                id: InstanceId::new(11)
            }
        ]
    );
    assert_eq!(scene.path(workspace).unwrap(), "/Game/World");
    assert_eq!(scene.path(audio).unwrap(), "/Game/World/Audio");
    assert_eq!(
        scene.children(workspace).unwrap(),
        &[audio, InstanceId::new(11)]
    );
}

#[test]
fn mutation_queue_deletes_subtrees_deterministically() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let parent = scene.add_child(workspace, "Workspace", "Parent").unwrap();
    let child = scene.add_child(parent, "Workspace", "Child").unwrap();

    let mut queue = SceneMutationQueue::new();
    queue.delete(parent);

    assert_eq!(
        scene.apply_mutations(queue).unwrap(),
        vec![SceneMutationResult::Deleted {
            id: parent,
            deleted_ids: vec![parent, child]
        }]
    );
    assert_eq!(scene.children(workspace).unwrap(), &[]);
    assert_eq!(
        scene.get(parent).unwrap_err(),
        SceneError::InvalidInstanceId { id: parent }
    );
    assert_eq!(
        scene.get(child).unwrap_err(),
        SceneError::InvalidInstanceId { id: child }
    );
}

#[test]
fn mutation_queue_duplicates_subtrees_deterministically() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let parent = scene.add_child(workspace, "Workspace", "Parent").unwrap();
    let child = scene.add_child(parent, "Part", "Child").unwrap();
    scene
        .set_property(
            child,
            "Visible",
            kinetik_reflect::PropertyValue::Bool(false),
        )
        .unwrap();

    let mut queue = SceneMutationQueue::new();
    queue.duplicate(parent, workspace);

    assert_eq!(
        scene.apply_mutations(queue).unwrap(),
        vec![SceneMutationResult::Duplicated {
            source_id: parent,
            new_root_id: InstanceId::new(5),
            duplicated_ids: vec![InstanceId::new(5), InstanceId::new(6)]
        }]
    );
    assert_eq!(
        scene.children(workspace).unwrap(),
        &[parent, InstanceId::new(5)]
    );
    assert_eq!(
        scene.path(InstanceId::new(5)).unwrap(),
        "/Game/Workspace/Parent"
    );
    assert_eq!(
        scene.path(InstanceId::new(6)).unwrap(),
        "/Game/Workspace/Parent/Child"
    );
    assert_ne!(
        scene.get(parent).unwrap().guid,
        scene.get(InstanceId::new(5)).unwrap().guid
    );
    assert_ne!(
        scene.get(child).unwrap().guid,
        scene.get(InstanceId::new(6)).unwrap().guid
    );
    assert_eq!(
        scene.get_property(InstanceId::new(6), "Visible").unwrap(),
        &kinetik_reflect::PropertyValue::Bool(false)
    );
}

#[test]
fn mutation_queue_rejects_invalid_handles_and_classes() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;

    let mut invalid_parent = SceneMutationQueue::new();
    invalid_parent.create_child(InstanceId::new(99), "Workspace", "MissingParent");
    assert_eq!(
        scene.apply_mutations(invalid_parent).unwrap_err(),
        SceneError::InvalidInstanceId {
            id: InstanceId::new(99)
        }
    );

    let mut unknown_class = SceneMutationQueue::new();
    unknown_class.create_child(workspace, "MissingClass", "Thing");
    assert_eq!(
        scene.apply_mutations(unknown_class).unwrap_err(),
        SceneError::UnknownClass {
            class_name: "MissingClass".to_owned()
        }
    );

    let mut invalid_child = SceneMutationQueue::new();
    invalid_child.reparent(InstanceId::new(99), workspace);
    assert_eq!(
        scene.apply_mutations(invalid_child).unwrap_err(),
        SceneError::InvalidInstanceId {
            id: InstanceId::new(99)
        }
    );

    let mut invalid_duplicate_source = SceneMutationQueue::new();
    invalid_duplicate_source.duplicate(InstanceId::new(99), workspace);
    assert_eq!(
        scene.apply_mutations(invalid_duplicate_source).unwrap_err(),
        SceneError::InvalidInstanceId {
            id: InstanceId::new(99)
        }
    );
}

#[test]
fn mutation_queue_rejects_root_and_cycle_operations() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let child = scene.add_child(workspace, "Workspace", "Child").unwrap();

    let mut delete_root = SceneMutationQueue::new();
    delete_root.delete(game);
    assert_eq!(
        scene.apply_mutations(delete_root).unwrap_err(),
        SceneError::CannotDeleteRoot { root_id: game }
    );

    let mut reparent_root = SceneMutationQueue::new();
    reparent_root.reparent(game, workspace);
    assert_eq!(
        scene.apply_mutations(reparent_root).unwrap_err(),
        SceneError::CannotReparentRoot { root_id: game }
    );

    let mut duplicate_root = SceneMutationQueue::new();
    duplicate_root.duplicate(game, workspace);
    assert_eq!(
        scene.apply_mutations(duplicate_root).unwrap_err(),
        SceneError::CannotDuplicateRoot { root_id: game }
    );

    let mut cycle = SceneMutationQueue::new();
    cycle.reparent(workspace, child);
    assert_eq!(
        scene.apply_mutations(cycle).unwrap_err(),
        SceneError::ReparentCycle {
            id: workspace,
            new_parent: child
        }
    );
}

#[test]
fn failed_mutation_queue_does_not_partially_apply() {
    let mut scene = Scene::new();
    let game = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
    let workspace = scene.add_child(game, "Workspace", "Workspace").unwrap();
    let child = scene.add_child(workspace, "Workspace", "Child").unwrap();

    let mut queue = SceneMutationQueue::new();
    queue.rename(workspace, "World");
    queue.reparent(workspace, child);

    assert_eq!(
        scene.apply_mutations(queue).unwrap_err(),
        SceneError::ReparentCycle {
            id: workspace,
            new_parent: child
        }
    );
    assert_eq!(scene.get(workspace).unwrap().name, "Workspace");
    assert_eq!(scene.path(child).unwrap(), "/Game/Workspace/Child");
}
