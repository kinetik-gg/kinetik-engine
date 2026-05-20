use crate::{
    rename_scene_instance, CommandError, CommandHistory, CommandTargetMode, DirtyStateExplanation,
    PropertyValueChange, UndoGroupId, RENAME_INSTANCE_COMMAND,
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
