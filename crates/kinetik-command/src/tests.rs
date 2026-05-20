use super::*;

use kinetik_core::{DiagnosticBlockingScope, DiagnosticSeverity};

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
fn command_result_success_stores_kind_mode_status_and_no_diagnostics() {
    let result = CommandResult::succeeded("CreateInstance", CommandTargetMode::Edit).unwrap();

    assert_eq!(result.command_kind(), "CreateInstance");
    assert_eq!(result.target_mode(), Some(CommandTargetMode::Edit));
    assert_eq!(result.status(), CommandStatus::Succeeded);
    assert!(result.is_success());
    assert!(!result.is_failure());
    assert!(result.diagnostics().is_empty());
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
