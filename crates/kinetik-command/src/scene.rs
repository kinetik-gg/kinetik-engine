use kinetik_core::InstanceId;
use kinetik_reflect::PropertyValue;
use kinetik_scene::{Scene, SceneMutationQueue, SceneMutationResult};

use crate::{
    ChangeTarget, CommandChangeRecord, CommandError, CommandModelResult, CommandResult,
    CommandTargetMode, PropertyValueChange,
};

/// Stable command kind for edit-mode scene instance rename.
pub const RENAME_INSTANCE_COMMAND: &str = "RenameInstance";

/// Stable command kind for edit-mode scene child creation.
pub const CREATE_INSTANCE_COMMAND: &str = "CreateInstance";

/// Stable command kind for edit-mode scene instance deletion.
pub const DELETE_INSTANCE_COMMAND: &str = "DeleteInstance";

/// Result of a successful scene child creation command.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneCreateCommandResult {
    /// Created scene instance ID.
    pub instance_id: InstanceId,
    /// Command result containing semantic change records.
    pub command: CommandResult,
}

/// Result of a successful scene instance deletion command.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneDeleteCommandResult {
    /// Deleted root scene instance ID.
    pub instance_id: InstanceId,
    /// Deleted instance IDs in deterministic parent-before-child order.
    pub deleted_ids: Vec<InstanceId>,
    /// Command result containing semantic change records.
    pub command: CommandResult,
}

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
        .map_err(|error| scene_validation_error(RENAME_INSTANCE_COMMAND, &error))?;
    let old_name = instance.name.clone();
    let guid = instance.guid;
    let old_path = scene
        .path(instance_id)
        .map_err(|error| scene_validation_error(RENAME_INSTANCE_COMMAND, &error))?;

    let mut queue = SceneMutationQueue::new();
    queue.rename(instance_id, new_name.clone());
    scene
        .apply_mutations(queue)
        .map_err(|error| scene_validation_error(RENAME_INSTANCE_COMMAND, &error))?;

    let new_path = scene
        .path(instance_id)
        .map_err(|error| scene_validation_error(RENAME_INSTANCE_COMMAND, &error))?;
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

/// Creates a child scene instance through the shared command result model.
///
/// # Errors
///
/// Returns [`CommandError`] when scene validation fails before mutation.
pub fn create_scene_child_instance(
    scene: &mut Scene,
    parent_id: InstanceId,
    class_name: impl Into<String>,
    name: impl Into<String>,
    document_path: impl Into<String>,
) -> CommandModelResult<SceneCreateCommandResult> {
    let class_name = class_name.into();
    let name = name.into();
    let document_path = document_path.into();
    scene
        .get(parent_id)
        .map_err(|error| scene_validation_error(CREATE_INSTANCE_COMMAND, &error))?;

    let mut queue = SceneMutationQueue::new();
    queue.create_child(parent_id, class_name.clone(), name.clone());
    let results = scene
        .apply_mutations(queue)
        .map_err(|error| scene_validation_error(CREATE_INSTANCE_COMMAND, &error))?;
    let instance_id = created_instance_id(&results)?;
    let instance = scene
        .get(instance_id)
        .map_err(|error| scene_validation_error(CREATE_INSTANCE_COMMAND, &error))?;
    let scene_path = scene
        .path(instance_id)
        .map_err(|error| scene_validation_error(CREATE_INSTANCE_COMMAND, &error))?;

    let change = CommandChangeRecord::new(
        CREATE_INSTANCE_COMMAND,
        CommandTargetMode::Edit,
        format!("created {scene_path}"),
    )?
    .with_targets(vec![ChangeTarget::Instance {
        guid: Some(instance.guid),
        scene_path: Some(scene_path),
    }])
    .with_property_value_change(PropertyValueChange::new(
        "Name",
        None,
        Some(PropertyValue::String(name)),
    ))
    .with_affected_documents(vec![document_path]);

    Ok(SceneCreateCommandResult {
        instance_id,
        command: CommandResult::succeeded_with_changes(
            CREATE_INSTANCE_COMMAND,
            CommandTargetMode::Edit,
            [change],
        )?,
    })
}

/// Deletes a scene instance subtree through the shared command result model.
///
/// # Errors
///
/// Returns [`CommandError`] when scene validation fails before mutation.
pub fn delete_scene_instance(
    scene: &mut Scene,
    instance_id: InstanceId,
    document_path: impl Into<String>,
) -> CommandModelResult<SceneDeleteCommandResult> {
    let document_path = document_path.into();
    let instance = scene
        .get(instance_id)
        .map_err(|error| scene_validation_error(DELETE_INSTANCE_COMMAND, &error))?;
    let name = instance.name.clone();
    let guid = instance.guid;
    let scene_path = scene
        .path(instance_id)
        .map_err(|error| scene_validation_error(DELETE_INSTANCE_COMMAND, &error))?;

    let mut queue = SceneMutationQueue::new();
    queue.delete(instance_id);
    let results = scene
        .apply_mutations(queue)
        .map_err(|error| scene_validation_error(DELETE_INSTANCE_COMMAND, &error))?;
    let deleted_ids = deleted_instance_ids(&results)?;

    let change = CommandChangeRecord::new(
        DELETE_INSTANCE_COMMAND,
        CommandTargetMode::Edit,
        format!("deleted {scene_path}"),
    )?
    .with_targets(vec![ChangeTarget::Instance {
        guid: Some(guid),
        scene_path: Some(scene_path),
    }])
    .with_property_value_change(PropertyValueChange::new(
        "Name",
        Some(PropertyValue::String(name)),
        None,
    ))
    .with_affected_documents(vec![document_path]);

    Ok(SceneDeleteCommandResult {
        instance_id,
        deleted_ids,
        command: CommandResult::succeeded_with_changes(
            DELETE_INSTANCE_COMMAND,
            CommandTargetMode::Edit,
            [change],
        )?,
    })
}

fn scene_validation_error(command_kind: &str, error: &kinetik_scene::SceneError) -> CommandError {
    CommandError::ValidationFailed {
        command_kind: command_kind.to_owned(),
        reason: error.to_string(),
    }
}

fn created_instance_id(results: &[SceneMutationResult]) -> CommandModelResult<InstanceId> {
    match results {
        [SceneMutationResult::Created { id }] => Ok(*id),
        _ => Err(CommandError::ValidationFailed {
            command_kind: CREATE_INSTANCE_COMMAND.to_owned(),
            reason: "scene create command did not produce one created instance".to_owned(),
        }),
    }
}

fn deleted_instance_ids(results: &[SceneMutationResult]) -> CommandModelResult<Vec<InstanceId>> {
    match results {
        [SceneMutationResult::Deleted { id: _, deleted_ids }] => Ok(deleted_ids.clone()),
        _ => Err(CommandError::ValidationFailed {
            command_kind: DELETE_INSTANCE_COMMAND.to_owned(),
            reason: "scene delete command did not produce one deleted subtree".to_owned(),
        }),
    }
}
