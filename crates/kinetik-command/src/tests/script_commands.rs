use kinetik_core::{InstanceGuid, InstanceId, ResourceId, ScriptId};
use kinetik_script::{ScriptAssetRef, ScriptAttachment, ScriptAttachmentTarget, ScriptLanguage};

use crate::{
    attach_instance_script, detach_instance_script, CommandError, CommandHistory,
    CommandTargetMode, DirtyStateExplanation, ScriptAttachmentDocument, UndoGroupId,
    ATTACH_SCRIPT_COMMAND, DETACH_SCRIPT_COMMAND,
};

#[test]
fn attach_instance_script_mutates_document_and_returns_semantic_change() {
    let mut document = ScriptAttachmentDocument::new();
    let attachment = player_attachment();

    let result =
        attach_instance_script(&mut document, attachment.clone(), "scenes/main.knscene").unwrap();

    assert_eq!(result.attachment_id, attachment.id);
    assert_eq!(document.attachments(), std::slice::from_ref(&attachment));
    assert_eq!(result.command.command_kind(), ATTACH_SCRIPT_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.command.diagnostics().is_empty());
    assert_eq!(result.command.changes().len(), 1);

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), ATTACH_SCRIPT_COMMAND);
    assert_eq!(change.target_mode(), CommandTargetMode::Edit);
    assert_eq!(change.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(
        change.dirty_summary(),
        "attached res://scripts/player.luau to /Game/Workspace/Player"
    );
    assert_eq!(change.targets().len(), 3);
}

#[test]
fn attach_instance_script_integrates_with_history_and_dirty_explanation() {
    let mut document = ScriptAttachmentDocument::new();
    let result =
        attach_instance_script(&mut document, player_attachment(), "scenes/main.knscene").unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Attach Player Script", &result.command)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["attached res://scripts/player.luau to /Game/Workspace/Player".to_owned()]
    );
    assert_eq!(explanation.changes()[0].undo_group(), UndoGroupId::new(1));
}

#[test]
fn attach_instance_script_rejects_duplicate_binding_without_mutation() {
    let mut document = ScriptAttachmentDocument::new();
    let first = player_attachment();
    attach_instance_script(&mut document, first, "scenes/main.knscene").unwrap();
    let duplicate = ScriptAttachment::new(
        ScriptId::new(99),
        ScriptAssetRef::new("res://scripts/player.luau", ScriptLanguage::luau()),
        ScriptAttachmentTarget::with_guid(InstanceId::new(10), InstanceGuid::new(20)),
    )
    .with_scene_path("/Game/Workspace/Player");

    let error =
        attach_instance_script(&mut document, duplicate, "scenes/main.knscene").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(ATTACH_SCRIPT_COMMAND));
    assert_eq!(document.attachments().len(), 1);
}

#[test]
fn attach_instance_script_rejects_invalid_script_path_without_mutation() {
    let mut document = ScriptAttachmentDocument::new();
    let invalid = ScriptAttachment::new(
        ScriptId::new(1),
        ScriptAssetRef::new("scripts/player.luau", ScriptLanguage::luau()),
        ScriptAttachmentTarget::runtime_only(InstanceId::new(10)),
    );

    let error = attach_instance_script(&mut document, invalid, "scenes/main.knscene").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(document.attachments().is_empty());
}

#[test]
fn detach_instance_script_mutates_document_and_returns_semantic_change() {
    let mut document = ScriptAttachmentDocument::new();
    let attachment = player_attachment();
    attach_instance_script(&mut document, attachment.clone(), "scenes/main.knscene").unwrap();

    let result =
        detach_instance_script(&mut document, attachment.id, "scenes/main.knscene").unwrap();

    assert_eq!(result.attachment, attachment);
    assert!(document.attachments().is_empty());
    assert_eq!(result.command.command_kind(), DETACH_SCRIPT_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), DETACH_SCRIPT_COMMAND);
    assert_eq!(
        change.dirty_summary(),
        "detached res://scripts/player.luau from /Game/Workspace/Player"
    );
    assert_eq!(change.targets().len(), 3);
}

#[test]
fn detach_instance_script_rejects_missing_attachment_without_mutation() {
    let mut document = ScriptAttachmentDocument::new();
    let attachment = player_attachment();
    attach_instance_script(&mut document, attachment.clone(), "scenes/main.knscene").unwrap();

    let error = detach_instance_script(&mut document, ScriptId::new(99), "scenes/main.knscene")
        .unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(DETACH_SCRIPT_COMMAND));
    assert_eq!(document.attachments(), &[attachment]);
}

fn player_attachment() -> ScriptAttachment {
    ScriptAttachment::new(
        ScriptId::new(1),
        ScriptAssetRef::new("res://scripts/player.luau", ScriptLanguage::luau())
            .with_resource_id(ResourceId::new(3)),
        ScriptAttachmentTarget::with_guid(InstanceId::new(10), InstanceGuid::new(20)),
    )
    .with_scene_path("/Game/Workspace/Player")
}
