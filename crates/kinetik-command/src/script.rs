use kinetik_script::{ScriptAttachment, ScriptAttachmentId};

use crate::{
    ChangeTarget, CommandChangeRecord, CommandError, CommandModelResult, CommandResult,
    CommandTargetMode,
};

/// Stable command kind for edit-mode script attachment.
pub const ATTACH_SCRIPT_COMMAND: &str = "AttachScript";

/// Stable command kind for edit-mode script detachment.
pub const DETACH_SCRIPT_COMMAND: &str = "DetachScript";

/// In-memory edit-mode script attachment collection for command execution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptAttachmentDocument {
    attachments: Vec<ScriptAttachment>,
}

impl ScriptAttachmentDocument {
    /// Creates an empty script attachment document.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            attachments: Vec::new(),
        }
    }

    /// Returns script attachments in deterministic insertion order.
    #[must_use]
    pub fn attachments(&self) -> &[ScriptAttachment] {
        &self.attachments
    }

    fn get(&self, id: ScriptAttachmentId) -> Option<&ScriptAttachment> {
        self.attachments
            .iter()
            .find(|attachment| attachment.id == id)
    }

    fn contains_same_binding(&self, attachment: &ScriptAttachment) -> bool {
        self.attachments.iter().any(|existing| {
            existing.target.instance_id == attachment.target.instance_id
                && existing.script.path == attachment.script.path
        })
    }

    fn push(&mut self, attachment: ScriptAttachment) {
        self.attachments.push(attachment);
    }

    fn remove(&mut self, id: ScriptAttachmentId) -> Option<ScriptAttachment> {
        let index = self
            .attachments
            .iter()
            .position(|attachment| attachment.id == id)?;
        Some(self.attachments.remove(index))
    }
}

/// Result of a successful script attach command.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptAttachCommandResult {
    /// Attached script ID.
    pub attachment_id: ScriptAttachmentId,
    /// Command result containing semantic change records.
    pub command: CommandResult,
}

/// Result of a successful script detach command.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptDetachCommandResult {
    /// Detached script attachment.
    pub attachment: ScriptAttachment,
    /// Command result containing semantic change records.
    pub command: CommandResult,
}

/// Attaches a script asset to an instance through the shared command result model.
///
/// # Errors
///
/// Returns [`CommandError`] when attachment validation fails before mutation.
pub fn attach_instance_script(
    document: &mut ScriptAttachmentDocument,
    attachment: ScriptAttachment,
    document_path: impl Into<String>,
) -> CommandModelResult<ScriptAttachCommandResult> {
    let document_path = document_path.into();
    validate_attachment(ATTACH_SCRIPT_COMMAND, &attachment)?;
    if document.get(attachment.id).is_some() {
        return Err(CommandError::ValidationFailed {
            command_kind: ATTACH_SCRIPT_COMMAND.to_owned(),
            reason: format!("script attachment ID already exists: {}", attachment.id),
        });
    }
    if document.contains_same_binding(&attachment) {
        return Err(CommandError::ValidationFailed {
            command_kind: ATTACH_SCRIPT_COMMAND.to_owned(),
            reason: format!(
                "script '{}' is already attached to {}",
                attachment.script.path,
                attachment_target_name(&attachment)
            ),
        });
    }

    let attachment_id = attachment.id;
    let change = attachment_change_record(ATTACH_SCRIPT_COMMAND, &attachment, &document_path)?;
    document.push(attachment);

    Ok(ScriptAttachCommandResult {
        attachment_id,
        command: CommandResult::succeeded_with_changes(
            ATTACH_SCRIPT_COMMAND,
            CommandTargetMode::Edit,
            [change],
        )?,
    })
}

/// Detaches a script asset from an instance through the shared command result model.
///
/// # Errors
///
/// Returns [`CommandError`] when the attachment ID is not present.
pub fn detach_instance_script(
    document: &mut ScriptAttachmentDocument,
    attachment_id: ScriptAttachmentId,
    document_path: impl Into<String>,
) -> CommandModelResult<ScriptDetachCommandResult> {
    let document_path = document_path.into();
    let Some(attachment) = document.get(attachment_id).cloned() else {
        return Err(CommandError::ValidationFailed {
            command_kind: DETACH_SCRIPT_COMMAND.to_owned(),
            reason: format!("script attachment ID is not present: {attachment_id}"),
        });
    };
    let change = attachment_change_record(DETACH_SCRIPT_COMMAND, &attachment, &document_path)?;
    let Some(removed) = document.remove(attachment_id) else {
        return Err(CommandError::ValidationFailed {
            command_kind: DETACH_SCRIPT_COMMAND.to_owned(),
            reason: format!("script attachment ID is not present: {attachment_id}"),
        });
    };

    Ok(ScriptDetachCommandResult {
        attachment: removed,
        command: CommandResult::succeeded_with_changes(
            DETACH_SCRIPT_COMMAND,
            CommandTargetMode::Edit,
            [change],
        )?,
    })
}

fn validate_attachment(
    command_kind: &str,
    attachment: &ScriptAttachment,
) -> CommandModelResult<()> {
    if attachment.script.path.trim().is_empty() {
        return Err(CommandError::ValidationFailed {
            command_kind: command_kind.to_owned(),
            reason: "script path must not be empty".to_owned(),
        });
    }
    if !attachment.script.path.starts_with("res://") || attachment.script.path == "res://" {
        return Err(CommandError::ValidationFailed {
            command_kind: command_kind.to_owned(),
            reason: format!(
                "script path must be a project resource path: {}",
                attachment.script.path
            ),
        });
    }
    if attachment.script.language.as_str().trim().is_empty() {
        return Err(CommandError::ValidationFailed {
            command_kind: command_kind.to_owned(),
            reason: "script language must not be empty".to_owned(),
        });
    }

    Ok(())
}

fn attachment_change_record(
    command_kind: &str,
    attachment: &ScriptAttachment,
    document_path: &str,
) -> CommandModelResult<CommandChangeRecord> {
    let action = if command_kind == ATTACH_SCRIPT_COMMAND {
        "attached"
    } else {
        "detached"
    };

    CommandChangeRecord::new(
        command_kind,
        CommandTargetMode::Edit,
        format!(
            "{action} {} {} {}",
            attachment.script.path,
            if action == "attached" { "to" } else { "from" },
            attachment_target_name(attachment)
        ),
    )
    .map(|record| {
        record
            .with_targets(vec![
                ChangeTarget::Instance {
                    guid: attachment.target.instance_guid,
                    scene_path: attachment.scene_path.clone(),
                },
                ChangeTarget::Script {
                    script_path: attachment.script.path.clone(),
                },
                ChangeTarget::Document {
                    document_path: document_path.to_owned(),
                },
            ])
            .with_affected_documents(vec![document_path.to_owned()])
    })
}

fn attachment_target_name(attachment: &ScriptAttachment) -> String {
    attachment
        .scene_path
        .clone()
        .unwrap_or_else(|| attachment.target.instance_id.to_string())
}
