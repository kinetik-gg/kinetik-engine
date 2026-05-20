use kinetik_resource::{AssetPath, AssetReference, ResourceError};

use crate::{
    ChangeTarget, CommandChangeRecord, CommandError, CommandModelResult, CommandResult,
    CommandTargetMode,
};

/// Stable command kind for an asset reimport request.
pub const REIMPORT_ASSET_COMMAND: &str = "ReimportAsset";

/// Stable command kind for updating an asset manifest path.
pub const UPDATE_ASSET_PATH_COMMAND: &str = "UpdateAssetPath";

/// Result of a successful asset reimport request command.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetReimportCommandResult {
    /// Asset requested for reimport.
    pub asset: AssetReference,
    /// Command result containing semantic change records.
    pub command: CommandResult,
}

/// Result of a successful asset path update command.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetPathCommandResult {
    /// Asset reference before the path update.
    pub old_asset: AssetReference,
    /// Asset reference after the path update.
    pub new_asset: AssetReference,
    /// Command result containing semantic change records.
    pub command: CommandResult,
}

/// Produces a validated asset reimport command result without running importers.
///
/// # Errors
///
/// Returns [`CommandError`] when the affected document path is empty.
pub fn request_asset_reimport(
    asset: AssetReference,
    document_path: impl Into<String>,
) -> CommandModelResult<AssetReimportCommandResult> {
    let document_path = validate_document_path(REIMPORT_ASSET_COMMAND, document_path.into())?;
    let change = CommandChangeRecord::new(
        REIMPORT_ASSET_COMMAND,
        CommandTargetMode::Edit,
        format!("requested reimport for {}", asset.path()),
    )?
    .with_targets(vec![
        asset_target(&asset),
        ChangeTarget::Document {
            document_path: document_path.clone(),
        },
    ])
    .with_affected_documents(vec![document_path]);

    Ok(AssetReimportCommandResult {
        asset,
        command: CommandResult::succeeded_with_changes(
            REIMPORT_ASSET_COMMAND,
            CommandTargetMode::Edit,
            [change],
        )?,
    })
}

/// Produces a validated asset manifest path update command result.
///
/// # Errors
///
/// Returns [`CommandError`] when the new asset path or affected document path is invalid.
pub fn update_asset_path(
    asset: AssetReference,
    new_path: impl Into<String>,
    document_path: impl Into<String>,
) -> CommandModelResult<AssetPathCommandResult> {
    let document_path = validate_document_path(UPDATE_ASSET_PATH_COMMAND, document_path.into())?;
    let new_path = AssetPath::new(new_path.into()).map_err(|error| {
        resource_validation_error(UPDATE_ASSET_PATH_COMMAND, "asset path update", &error)
    })?;
    let new_asset = AssetReference::new(asset.guid(), new_path);

    let change = CommandChangeRecord::new(
        UPDATE_ASSET_PATH_COMMAND,
        CommandTargetMode::Edit,
        format!("moved asset {} to {}", asset.path(), new_asset.path()),
    )?
    .with_targets(vec![
        asset_target(&asset),
        asset_target(&new_asset),
        ChangeTarget::Document {
            document_path: document_path.clone(),
        },
    ])
    .with_affected_documents(vec![document_path]);

    Ok(AssetPathCommandResult {
        old_asset: asset,
        new_asset,
        command: CommandResult::succeeded_with_changes(
            UPDATE_ASSET_PATH_COMMAND,
            CommandTargetMode::Edit,
            [change],
        )?,
    })
}

fn validate_document_path(command_kind: &str, document_path: String) -> CommandModelResult<String> {
    if document_path.trim().is_empty() {
        return Err(CommandError::ValidationFailed {
            command_kind: command_kind.to_owned(),
            reason: "affected document path must not be empty".to_owned(),
        });
    }

    Ok(document_path)
}

fn asset_target(asset: &AssetReference) -> ChangeTarget {
    ChangeTarget::Asset {
        asset_guid: Some(asset.guid().raw().to_string()),
        asset_path: asset.path().as_str().to_owned(),
    }
}

fn resource_validation_error(
    command_kind: &str,
    context: &str,
    error: &ResourceError,
) -> CommandError {
    CommandError::ValidationFailed {
        command_kind: command_kind.to_owned(),
        reason: format!("{context} failed validation: {error}"),
    }
}
