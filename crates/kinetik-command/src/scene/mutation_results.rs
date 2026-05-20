use kinetik_core::InstanceId;
use kinetik_scene::SceneMutationResult;

use crate::{
    CommandError, CommandModelResult, CREATE_INSTANCE_COMMAND, DELETE_INSTANCE_COMMAND,
    DUPLICATE_INSTANCE_COMMAND, REPARENT_INSTANCE_COMMAND,
};

pub(super) fn created_instance_id(
    results: &[SceneMutationResult],
) -> CommandModelResult<InstanceId> {
    match results {
        [SceneMutationResult::Created { id }] => Ok(*id),
        _ => Err(CommandError::ValidationFailed {
            command_kind: CREATE_INSTANCE_COMMAND.to_owned(),
            reason: "scene create command did not produce one created instance".to_owned(),
        }),
    }
}

pub(super) fn deleted_instance_ids(
    results: &[SceneMutationResult],
) -> CommandModelResult<Vec<InstanceId>> {
    match results {
        [SceneMutationResult::Deleted { id: _, deleted_ids }] => Ok(deleted_ids.clone()),
        _ => Err(CommandError::ValidationFailed {
            command_kind: DELETE_INSTANCE_COMMAND.to_owned(),
            reason: "scene delete command did not produce one deleted subtree".to_owned(),
        }),
    }
}

pub(super) fn reparented_old_parent(
    results: &[SceneMutationResult],
) -> CommandModelResult<Option<InstanceId>> {
    match results {
        [SceneMutationResult::Reparented {
            id: _,
            old_parent,
            new_parent: _,
        }] => Ok(*old_parent),
        _ => Err(CommandError::ValidationFailed {
            command_kind: REPARENT_INSTANCE_COMMAND.to_owned(),
            reason: "scene reparent command did not produce one moved instance".to_owned(),
        }),
    }
}

pub(super) fn duplicated_instance_ids(
    results: &[SceneMutationResult],
) -> CommandModelResult<(InstanceId, Vec<InstanceId>)> {
    match results {
        [SceneMutationResult::Duplicated {
            source_id: _,
            new_root_id,
            duplicated_ids,
        }] => Ok((*new_root_id, duplicated_ids.clone())),
        _ => Err(CommandError::ValidationFailed {
            command_kind: DUPLICATE_INSTANCE_COMMAND.to_owned(),
            reason: "scene duplicate command did not produce one duplicated subtree".to_owned(),
        }),
    }
}
