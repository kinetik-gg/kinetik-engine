use super::*;

use kinetik_core::{DiagnosticBlockingScope, DiagnosticSeverity, InstanceGuid};
use kinetik_reflect::PropertyValue;

#[test]
fn exposes_crate_name() {
    assert_eq!(crate_name(), "kinetik-command");
}

#[test]
fn command_error_diagnostic_codes_are_stable() {
    assert_eq!(
        CommandError::EMPTY_COMMAND_KIND_CODE.as_str(),
        "KT_COMMAND_EMPTY_KIND"
    );
    assert_eq!(
        CommandError::EMPTY_DIRTY_SUMMARY_CODE.as_str(),
        "KT_COMMAND_EMPTY_DIRTY_SUMMARY"
    );
    assert_eq!(
        CommandError::AMBIGUOUS_TARGET_MODE_CODE.as_str(),
        "KT_COMMAND_AMBIGUOUS_TARGET_MODE"
    );
    assert_eq!(
        CommandError::WRONG_TARGET_MODE_CODE.as_str(),
        "KT_COMMAND_WRONG_TARGET_MODE"
    );
    assert_eq!(CommandError::COMMAND_SOURCE.as_str(), "Command");
}

#[test]
fn command_errors_convert_to_edit_blocking_diagnostics() {
    let diagnostic = CommandError::AmbiguousTargetMode {
        command_kind: "SetProperty".to_owned(),
    }
    .to_diagnostic();

    assert_eq!(diagnostic.code, CommandError::AMBIGUOUS_TARGET_MODE_CODE);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.source, CommandError::COMMAND_SOURCE);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Edit));
    assert!(diagnostic.message.contains("SetProperty"));
}

#[test]
fn undo_group_id_rejects_zero_and_displays_stably() {
    assert_eq!(UndoGroupId::new(7).raw(), 7);
    assert_eq!(UndoGroupId::new(7).to_string(), "UndoGroupId(7)");
}

#[test]
#[should_panic(expected = "UndoGroupId raw value must be non-zero")]
fn undo_group_id_rejects_zero() {
    let _ = UndoGroupId::new(0);
}

#[test]
fn change_record_stores_semantic_targets_values_documents_and_summary() {
    let value_change = PropertyValueChange::new(
        "Transform.Position",
        Some(PropertyValue::String("old".to_owned())),
        Some(PropertyValue::String("new".to_owned())),
    );
    let record = CommandChangeRecord::new(
        "SetProperty",
        CommandTargetMode::Edit,
        "/Game/Workspace.Part Transform.Position changed",
    )
    .unwrap()
    .with_targets(vec![
        ChangeTarget::Instance {
            guid: Some(InstanceGuid::new(7)),
            scene_path: Some("/Game/Workspace/Part".to_owned()),
        },
        ChangeTarget::Property {
            instance_guid: Some(InstanceGuid::new(7)),
            scene_path: Some("/Game/Workspace/Part".to_owned()),
            property_path: "Transform.Position".to_owned(),
        },
        ChangeTarget::Asset {
            asset_guid: Some("asset-guid".to_owned()),
            asset_path: "res://assets/part.glb".to_owned(),
        },
        ChangeTarget::Script {
            script_path: "scripts/part.luau".to_owned(),
        },
        ChangeTarget::Document {
            document_path: "scenes/main.knscene".to_owned(),
        },
    ])
    .with_property_value_change(value_change.clone())
    .with_affected_documents(vec!["scenes/main.knscene".to_owned()])
    .with_undo_group(UndoGroupId::new(3));

    assert_eq!(record.command_kind(), "SetProperty");
    assert_eq!(record.target_mode(), CommandTargetMode::Edit);
    assert_eq!(record.targets().len(), 5);
    assert_eq!(record.property_value_change(), Some(&value_change));
    assert_eq!(record.affected_documents(), &["scenes/main.knscene"]);
    assert_eq!(record.undo_group(), Some(UndoGroupId::new(3)));
    assert_eq!(
        record.dirty_summary(),
        "/Game/Workspace.Part Transform.Position changed"
    );
}

#[test]
fn change_record_rejects_empty_required_text() {
    assert_eq!(
        CommandChangeRecord::new(" ", CommandTargetMode::Edit, "changed").unwrap_err(),
        CommandError::EmptyCommandKind
    );
    assert_eq!(
        CommandChangeRecord::new("SetProperty", CommandTargetMode::Edit, " ").unwrap_err(),
        CommandError::EmptyDirtySummary
    );
}

#[test]
fn command_result_success_stores_kind_mode_status_and_no_diagnostics() {
    let result = CommandResult::succeeded("CreateInstance", CommandTargetMode::Edit).unwrap();

    assert_eq!(result.command_kind(), "CreateInstance");
    assert_eq!(result.target_mode(), Some(CommandTargetMode::Edit));
    assert_eq!(result.status(), CommandStatus::Succeeded);
    assert!(result.is_success());
    assert!(!result.is_failure());
    assert!(result.diagnostics().is_empty());
    assert!(result.changes().is_empty());
}

#[test]
fn command_result_success_preserves_change_record_order() {
    let first = CommandChangeRecord::new("RenameInstance", CommandTargetMode::Edit, "A renamed")
        .unwrap()
        .with_affected_documents(vec!["scenes/main.knscene".to_owned()]);
    let second = CommandChangeRecord::new("RenameInstance", CommandTargetMode::Edit, "B renamed")
        .unwrap()
        .with_affected_documents(vec!["scenes/main.knscene".to_owned()]);

    let result = CommandResult::succeeded_with_changes(
        "RenameInstance",
        CommandTargetMode::Edit,
        [first.clone(), second.clone()],
    )
    .unwrap();

    assert_eq!(result.status(), CommandStatus::Succeeded);
    assert!(result.diagnostics().is_empty());
    assert_eq!(result.changes(), &[first, second]);
}

#[test]
fn command_result_failure_preserves_diagnostic_order() {
    let first = CommandError::AmbiguousTargetMode {
        command_kind: "SetProperty".to_owned(),
    }
    .to_diagnostic();
    let second = CommandError::WrongTargetMode {
        command_kind: "PlayStep".to_owned(),
        expected: CommandTargetMode::Play,
        actual: CommandTargetMode::Edit,
    }
    .to_diagnostic();

    let result =
        CommandResult::failed("SetProperty", None, [first.clone(), second.clone()]).unwrap();

    assert_eq!(result.status(), CommandStatus::Failed);
    assert!(!result.is_success());
    assert!(result.is_failure());
    assert_eq!(result.diagnostics(), &[first, second]);
    assert!(result.changes().is_empty());
}

#[test]
fn command_result_rejected_converts_error_to_failure_diagnostic() {
    let error = CommandError::WrongTargetMode {
        command_kind: "PlayStep".to_owned(),
        expected: CommandTargetMode::Play,
        actual: CommandTargetMode::Edit,
    };

    let result =
        CommandResult::rejected("PlayStep", Some(CommandTargetMode::Edit), &error).unwrap();

    assert_eq!(result.status(), CommandStatus::Failed);
    assert_eq!(result.target_mode(), Some(CommandTargetMode::Edit));
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code,
        CommandError::WRONG_TARGET_MODE_CODE
    );
}

#[test]
fn command_result_rejects_empty_command_kind() {
    assert_eq!(
        CommandResult::succeeded(" ", CommandTargetMode::Edit).unwrap_err(),
        CommandError::EmptyCommandKind
    );
}

#[test]
fn target_mode_validation_rejects_ambiguous_edit_play_commands() {
    assert_eq!(
        require_target_mode("SetProperty", None).unwrap_err(),
        CommandError::AmbiguousTargetMode {
            command_kind: "SetProperty".to_owned()
        }
    );
}

#[test]
fn target_mode_validation_rejects_wrong_mode() {
    assert_eq!(
        require_specific_target_mode(
            "PlayStep",
            Some(CommandTargetMode::Edit),
            CommandTargetMode::Play,
        )
        .unwrap_err(),
        CommandError::WrongTargetMode {
            command_kind: "PlayStep".to_owned(),
            expected: CommandTargetMode::Play,
            actual: CommandTargetMode::Edit,
        }
    );
}

#[test]
fn target_mode_validation_accepts_expected_mode() {
    assert_eq!(
        require_specific_target_mode(
            "PlayStep",
            Some(CommandTargetMode::Play),
            CommandTargetMode::Play,
        )
        .unwrap(),
        CommandTargetMode::Play
    );
}
