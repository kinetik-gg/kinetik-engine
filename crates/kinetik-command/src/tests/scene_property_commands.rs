use crate::{
    create_scene_child_instance, set_scene_instance_property, CommandError, CommandHistory,
    CommandTargetMode, DirtyStateExplanation, PropertyValueChange, UndoGroupId,
    SET_PROPERTY_COMMAND,
};
use kinetik_reflect::PropertyValue;
use kinetik_scene::Scene;

#[test]
fn set_scene_instance_property_mutates_scene_and_returns_semantic_change() {
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

    let result = set_scene_instance_property(
        &mut scene,
        child,
        "Visible",
        PropertyValue::Bool(false),
        "scenes/main.knscene",
    )
    .unwrap();

    assert_eq!(result.instance_id, child);
    assert_eq!(result.property_path, "Visible");
    assert_eq!(result.old_value, PropertyValue::Bool(true));
    assert_eq!(result.new_value, PropertyValue::Bool(false));
    assert_eq!(
        scene.get_property(child, "Visible").unwrap(),
        &PropertyValue::Bool(false)
    );
    assert_eq!(result.command.command_kind(), SET_PROPERTY_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.command.diagnostics().is_empty());

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), SET_PROPERTY_COMMAND);
    assert_eq!(change.target_mode(), CommandTargetMode::Edit);
    assert_eq!(change.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(change.dirty_summary(), "set /Game/Workspace/Block.Visible");
    assert_eq!(
        change.property_value_change(),
        Some(&PropertyValueChange::new(
            "Visible",
            Some(PropertyValue::Bool(true)),
            Some(PropertyValue::Bool(false)),
        ))
    );
    assert_eq!(change.targets().len(), 2);
}

#[test]
fn set_scene_instance_property_updates_name_paths() {
    let mut scene = Scene::default_scene().unwrap();
    let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;

    let result = set_scene_instance_property(
        &mut scene,
        workspace,
        "Name",
        PropertyValue::String("World".to_owned()),
        "scenes/main.knscene",
    )
    .unwrap();

    assert_eq!(scene.path(workspace).unwrap(), "/Game/World");
    assert_eq!(
        scene.get_property(workspace, "Name").unwrap(),
        &PropertyValue::String("World".to_owned())
    );
    assert_eq!(
        result.command.changes()[0].dirty_summary(),
        "set /Game/Workspace.Name"
    );
}

#[test]
fn set_scene_instance_property_integrates_with_history_and_dirty_explanation() {
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
    let result = set_scene_instance_property(
        &mut scene,
        child,
        "Visible",
        PropertyValue::Bool(false),
        "scenes/main.knscene",
    )
    .unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Hide Block", &result.command)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["set /Game/Workspace/Block.Visible".to_owned()]
    );
}

#[test]
fn set_scene_instance_property_rejects_invalid_type_without_mutating_scene() {
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
    let original_value = scene.get_property(child, "Visible").unwrap().clone();

    let error = set_scene_instance_property(
        &mut scene,
        child,
        "Visible",
        PropertyValue::String("false".to_owned()),
        "scenes/main.knscene",
    )
    .unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(SET_PROPERTY_COMMAND));
    assert_eq!(
        scene.get_property(child, "Visible").unwrap(),
        &original_value
    );
}
