use kinetik_resource::{AssetGuid, AssetPath, AssetReference};

use crate::{
    request_asset_reimport, update_asset_path, CommandError, CommandHistory, CommandTargetMode,
    DirtyStateExplanation, UndoGroupId, REIMPORT_ASSET_COMMAND, UPDATE_ASSET_PATH_COMMAND,
};

#[test]
fn request_asset_reimport_returns_semantic_change_without_importing() {
    let asset = test_asset();

    let result = request_asset_reimport(asset.clone(), "assets.knmanifest").unwrap();

    assert_eq!(result.asset, asset);
    assert_eq!(result.command.command_kind(), REIMPORT_ASSET_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));
    assert!(result.command.diagnostics().is_empty());
    assert_eq!(result.command.changes().len(), 1);

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), REIMPORT_ASSET_COMMAND);
    assert_eq!(change.affected_documents(), &["assets.knmanifest"]);
    assert_eq!(
        change.dirty_summary(),
        "requested reimport for res://assets/models/tree.glb"
    );
    assert_eq!(change.targets().len(), 2);
}

#[test]
fn asset_reimport_integrates_with_history_and_dirty_explanation() {
    let result = request_asset_reimport(test_asset(), "assets.knmanifest").unwrap();
    let mut history = CommandHistory::new();

    let record = history
        .commit_result("Reimport Tree", &result.command)
        .unwrap()
        .unwrap();
    let explanation = DirtyStateExplanation::from_history(&history);

    assert_eq!(record.group_id(), UndoGroupId::new(1));
    assert_eq!(explanation.documents().len(), 1);
    assert_eq!(
        explanation.documents()[0].summaries(),
        &["requested reimport for res://assets/models/tree.glb".to_owned()]
    );
}

#[test]
fn update_asset_path_returns_manifest_intent_change() {
    let old_asset = test_asset();

    let result = update_asset_path(
        old_asset.clone(),
        "res://assets/trees/oak.glb",
        "assets.knmanifest",
    )
    .unwrap();

    assert_eq!(result.old_asset, old_asset);
    assert_eq!(result.new_asset.guid(), AssetGuid::new(7));
    assert_eq!(
        result.new_asset.path().as_str(),
        "res://assets/trees/oak.glb"
    );
    assert_eq!(result.command.command_kind(), UPDATE_ASSET_PATH_COMMAND);
    assert_eq!(result.command.target_mode(), Some(CommandTargetMode::Edit));

    let change = &result.command.changes()[0];
    assert_eq!(change.command_kind(), UPDATE_ASSET_PATH_COMMAND);
    assert_eq!(change.affected_documents(), &["assets.knmanifest"]);
    assert_eq!(
        change.dirty_summary(),
        "moved asset res://assets/models/tree.glb to res://assets/trees/oak.glb"
    );
    assert_eq!(change.targets().len(), 3);
}

#[test]
fn update_asset_path_rejects_invalid_paths() {
    let error =
        update_asset_path(test_asset(), "assets/trees/oak.glb", "assets.knmanifest").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(UPDATE_ASSET_PATH_COMMAND));
}

#[test]
fn asset_commands_reject_empty_document_paths() {
    let error = request_asset_reimport(test_asset(), " ").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        CommandError::VALIDATION_FAILED_CODE
    );
    assert!(error.to_string().contains(REIMPORT_ASSET_COMMAND));
}

fn test_asset() -> AssetReference {
    AssetReference::new(
        AssetGuid::new(7),
        AssetPath::new("res://assets/models/tree.glb").unwrap(),
    )
}
