use crate::{
    create_scene_child_instance, delete_scene_instance, duplicate_scene_instance,
    rename_scene_instance, reparent_scene_instance, CommandError, CommandHistory,
    CommandTargetMode, DirtyStateExplanation, PropertyValueChange, UndoGroupId,
    CREATE_INSTANCE_COMMAND, DELETE_INSTANCE_COMMAND, DUPLICATE_INSTANCE_COMMAND,
    RENAME_INSTANCE_COMMAND, REPARENT_INSTANCE_COMMAND,
};
use kinetik_reflect::PropertyValue;
use kinetik_scene::Scene;

#[test]
fn rename_scene_instance_mutates_scene_and_returns_semantic_change() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;

    let result =
        rename_scene_instance(&mut scene, workspace, "World", "scenes/main.knscene").unwrap();

    assert!(result.is_success());
    assert_eq!(scene.path(workspace).unwrap(), "/Game/World");
    assert_eq!(
        scene.get_property(workspace, "Name").unwrap(),
        &PropertyValue::String("World".to_owned())
    );
    assert_eq!(result.command_kind(), RENAME_INSTANCE_COMMAND);
    assert_eq!(result.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.diagnostics().is_empty());
    assert_eq!(result.changes().len(), 1);

    let change = &result.changes()[0];
    assert_eq!(change.command_kind(), RENAME_INSTANCE_COMMAND);
    assert_eq!(change.target_mode(), CommandTargetMode::Edit);
    assert_eq!(change.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(change.dirty_summary(), "/Game/Workspace renamed to World");
    assert_eq!(
        change.property_value_change(),
        Some(&PropertyValueChange::new(
            "Name",
            Some(PropertyValue::String("Workspace".to_owned())),
            Some(PropertyValue::String("World".to_owned())),
        ))
    );
    assert_eq!(change.targets().len(), 2);
}

#[test]
fn rename_scene_instance_integrates_with_history_and_dirty_explanation() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let result =
        rename_scene_instance(&mut scene, workspace, "World", "scenes/main.knscene").unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Rename Workspace", &result)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].document_path(),
        "scenes/main.knscene"
    );
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["/Game/Workspace renamed to World".to_owned()]
    );
    assert_eq!(explanation.changes()[0].undo_group(), UndoGroupId::new(1));
}

#[test]
fn rename_scene_instance_rejects_invalid_name_without_mutating_scene() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;

    let error = rename_scene_instance(&mut scene, workspace, "Bad/Name", "scenes/main.knscene")
        .unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(RENAME_INSTANCE_COMMAND));
    assert_eq!(scene.path(workspace).unwrap(), "/Game/Workspace");
    assert_eq!(
        scene.get_property(workspace, "Name").unwrap(),
        &PropertyValue::String("Workspace".to_owned())
    );
}

#[test]
fn create_scene_child_instance_mutates_scene_and_returns_semantic_change() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;

    let result = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap();

    assert_eq!(
        scene.path(result.instance_id).unwrap(),
        "/Game/Workspace/Block"
    );
    assert_eq!(result.command.command_kind(), CREATE_INSTANCE_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.command.diagnostics().is_empty());
    assert_eq!(result.command.changes().len(), 1);

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), CREATE_INSTANCE_COMMAND);
    assert_eq!(change.target_mode(), CommandTargetMode::Edit);
    assert_eq!(change.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(change.dirty_summary(), "created /Game/Workspace/Block");
    assert_eq!(
        change.property_value_change(),
        Some(&PropertyValueChange::new(
            "Name",
            None,
            Some(PropertyValue::String("Block".to_owned())),
        ))
    );
    assert_eq!(change.targets().len(), 1);
}

#[test]
fn create_scene_child_instance_integrates_with_history_and_dirty_explanation() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let result = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Create Block", &result.command)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["created /Game/Workspace/Block".to_owned()]
    );
}

#[test]
fn create_scene_child_instance_rejects_unknown_class_without_mutating_scene() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let original_children = scene.children(workspace).unwrap().to_vec();

    let error = create_scene_child_instance(
        &mut scene,
        workspace,
        "MissingClass",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(CREATE_INSTANCE_COMMAND));
    assert_eq!(scene.children(workspace).unwrap(), original_children);
}

#[test]
fn delete_scene_instance_mutates_scene_and_returns_semantic_change() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let child = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap()
    .instance_id;

    let result = delete_scene_instance(&mut scene, child, "scenes/main.knscene").unwrap();

    assert_eq!(result.instance_id, child);
    assert_eq!(result.deleted_ids, vec![child]);
    assert!(scene.get(child).is_err());
    assert_eq!(result.command.command_kind(), DELETE_INSTANCE_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.command.diagnostics().is_empty());
    assert_eq!(result.command.changes().len(), 1);

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), DELETE_INSTANCE_COMMAND);
    assert_eq!(change.target_mode(), CommandTargetMode::Edit);
    assert_eq!(change.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(change.dirty_summary(), "deleted /Game/Workspace/Block");
    assert_eq!(
        change.property_value_change(),
        Some(&PropertyValueChange::new(
            "Name",
            Some(PropertyValue::String("Block".to_owned())),
            None,
        ))
    );
    assert_eq!(change.targets().len(), 1);
}

#[test]
fn delete_scene_instance_integrates_with_history_and_dirty_explanation() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let child = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap()
    .instance_id;
    let result = delete_scene_instance(&mut scene, child, "scenes/main.knscene").unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Delete Block", &result.command)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["deleted /Game/Workspace/Block".to_owned()]
    );
}

#[test]
fn delete_scene_instance_rejects_root_without_mutating_scene() {
    let mut scene = Scene::default_scene().unwrap();
    let root = scene.get_by_path("/Game").unwrap().id;
    let original_path = scene.path(root).unwrap();

    let error = delete_scene_instance(&mut scene, root, "scenes/main.knscene").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(DELETE_INSTANCE_COMMAND));
    assert_eq!(scene.path(root).unwrap(), original_path);
}

#[test]
fn reparent_scene_instance_mutates_scene_and_returns_semantic_change() {
    let mut scene = Scene::default_scene().unwrap();
    let game = scene.get_by_path("/Game").unwrap().id;
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let child = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap()
    .instance_id;

    let result = reparent_scene_instance(&mut scene, child, game, "scenes/main.knscene").unwrap();

    assert_eq!(result.instance_id, child);
    assert_eq!(result.old_parent, Some(workspace));
    assert_eq!(result.new_parent, game);
    assert_eq!(scene.path(child).unwrap(), "/Game/Block");
    assert_eq!(result.command.command_kind(), REPARENT_INSTANCE_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.command.diagnostics().is_empty());
    assert_eq!(result.command.changes().len(), 1);

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), REPARENT_INSTANCE_COMMAND);
    assert_eq!(change.target_mode(), CommandTargetMode::Edit);
    assert_eq!(change.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(
        change.dirty_summary(),
        "moved /Game/Workspace/Block to /Game/Block"
    );
    assert_eq!(change.targets().len(), 1);
}

#[test]
fn reparent_scene_instance_integrates_with_history_and_dirty_explanation() {
    let mut scene = Scene::default_scene().unwrap();
    let game = scene.get_by_path("/Game").unwrap().id;
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let child = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap()
    .instance_id;
    let result = reparent_scene_instance(&mut scene, child, game, "scenes/main.knscene").unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Move Block", &result.command)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["moved /Game/Workspace/Block to /Game/Block".to_owned()]
    );
}

#[test]
fn reparent_scene_instance_rejects_cycles_without_mutating_scene() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let child = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap()
    .instance_id;
    let original_path = scene.path(workspace).unwrap();

    let error =
        reparent_scene_instance(&mut scene, workspace, child, "scenes/main.knscene").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(REPARENT_INSTANCE_COMMAND));
    assert_eq!(scene.path(workspace).unwrap(), original_path);
    assert_eq!(
        scene.path(child).unwrap(),
        "/Game/Workspace/Block".to_owned()
    );
}

#[test]
fn duplicate_scene_instance_mutates_scene_and_returns_semantic_change() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let parent = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap()
    .instance_id;
    let child =
        create_scene_child_instance(&mut scene, parent, "Part", "Nested", "scenes/main.knscene")
            .unwrap()
            .instance_id;
    scene
        .set_property(child, "Visible", PropertyValue::Bool(false))
        .unwrap();

    let result =
        duplicate_scene_instance(&mut scene, parent, workspace, "scenes/main.knscene").unwrap();

    assert_eq!(result.source_id, parent);
    assert_eq!(result.duplicated_ids.len(), 2);
    assert_eq!(result.duplicated_ids[0], result.new_root_id);
    assert_eq!(
        scene.path(result.new_root_id).unwrap(),
        "/Game/Workspace/Block"
    );
    let duplicated_child = scene.children(result.new_root_id).unwrap()[0];
    assert_eq!(duplicated_child, result.duplicated_ids[1]);
    assert_eq!(
        scene.path(duplicated_child).unwrap(),
        "/Game/Workspace/Block/Nested"
    );
    assert_ne!(
        scene.get(parent).unwrap().guid,
        scene.get(result.new_root_id).unwrap().guid
    );
    assert_eq!(
        scene.get_property(duplicated_child, "Visible").unwrap(),
        &PropertyValue::Bool(false)
    );
    assert_eq!(result.command.command_kind(), DUPLICATE_INSTANCE_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.command.diagnostics().is_empty());
    assert_eq!(result.command.changes().len(), 1);

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), DUPLICATE_INSTANCE_COMMAND);
    assert_eq!(change.target_mode(), CommandTargetMode::Edit);
    assert_eq!(change.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(
        change.dirty_summary(),
        "duplicated /Game/Workspace/Block to /Game/Workspace/Block"
    );
    assert_eq!(
        change.property_value_change(),
        Some(&PropertyValueChange::new(
            "Name",
            None,
            Some(PropertyValue::String("Block".to_owned())),
        ))
    );
    assert_eq!(change.targets().len(), 2);
}

#[test]
fn duplicate_scene_instance_integrates_with_history_and_dirty_explanation() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let child = create_scene_child_instance(
        &mut scene,
        workspace,
        "Part",
        "Block",
        "scenes/main.knscene",
    )
    .unwrap()
    .instance_id;
    let result =
        duplicate_scene_instance(&mut scene, child, workspace, "scenes/main.knscene").unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Duplicate Block", &result.command)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["duplicated /Game/Workspace/Block to /Game/Workspace/Block".to_owned()]
    );
}

#[test]
fn duplicate_scene_instance_rejects_root_without_mutating_scene() {
    let mut scene = Scene::default_scene().unwrap();
    let root = scene.get_by_path("/Game").unwrap().id;
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
    let original_children = scene.children(workspace).unwrap().to_vec();

    let error =
        duplicate_scene_instance(&mut scene, root, workspace, "scenes/main.knscene").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(DUPLICATE_INSTANCE_COMMAND));
    assert_eq!(scene.children(workspace).unwrap(), original_children);
}
