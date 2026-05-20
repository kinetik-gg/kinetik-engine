use kinetik_core::InstanceId;
use kinetik_reflect::PropertyValue;
use kinetik_scene::{Scene, SceneMutationQueue};

use crate::{
    ChangeTarget, CommandChangeRecord, CommandError, CommandModelResult, CommandResult,
    CommandTargetMode, PropertyValueChange,
};

/// Stable command kind for edit-mode scene instance rename.
pub const RENAME_INSTANCE_COMMAND: &str = "RenameInstance";

/// Renames a scene instance through the shared command result model.
///
/// # Errors
///
/// Returns [`CommandError`] when scene validation fails before mutation.
pub fn rename_scene_instance(
    scene: &mut Scene,
    instance_id: InstanceId,
    new_name: impl Into<String>,
    document_path: impl Into<String>,
) -> CommandModelResult<CommandResult> {
    let document_path = document_path.into();
    let new_name = new_name.into();
    let instance = scene
        .get(instance_id)
        .map_err(|error| scene_validation_error(&error))?;
    let old_name = instance.name.clone();
    let guid = instance.guid;
    let old_path = scene
        .path(instance_id)
        .map_err(|error| scene_validation_error(&error))?;

    let mut queue = SceneMutationQueue::new();
    queue.rename(instance_id, new_name.clone());
    scene
        .apply_mutations(queue)
        .map_err(|error| scene_validation_error(&error))?;

    let new_path = scene
        .path(instance_id)
        .map_err(|error| scene_validation_error(&error))?;
    let change = CommandChangeRecord::new(
        RENAME_INSTANCE_COMMAND,
        CommandTargetMode::Edit,
        format!("{old_path} renamed to {new_name}"),
    )?
    .with_targets(vec![
        ChangeTarget::Instance {
            guid: Some(guid),
            scene_path: Some(new_path.clone()),
        },
        ChangeTarget::Property {
            instance_guid: Some(guid),
            scene_path: Some(new_path),
            property_path: "Name".to_owned(),
        },
    ])
    .with_property_value_change(PropertyValueChange::new(
        "Name",
        Some(PropertyValue::String(old_name)),
        Some(PropertyValue::String(new_name)),
    ))
    .with_affected_documents(vec![document_path]);

    CommandResult::succeeded_with_changes(
        RENAME_INSTANCE_COMMAND,
        CommandTargetMode::Edit,
        [change],
    )
}

fn scene_validation_error(error: &kinetik_scene::SceneError) -> CommandError {
    CommandError::ValidationFailed {
        command_kind: RENAME_INSTANCE_COMMAND.to_owned(),
        reason: error.to_string(),
    }
}
