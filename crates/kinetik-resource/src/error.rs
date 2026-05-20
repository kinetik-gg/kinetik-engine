use core::fmt;

use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticLocation, DiagnosticSeverity,
    DiagnosticSource,
};

use crate::{AssetGuid, AssetPath};

/// Result type for resource model operations.
pub type ResourceResult<T> = Result<T, ResourceError>;

/// Errors returned by resource identity and path validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// Stable asset GUID raw value was zero.
    InvalidAssetGuid {
        /// Invalid raw GUID value.
        raw: u64,
    },
    /// Asset path was empty.
    EmptyAssetPath,
    /// Asset path did not follow the `res://` project path contract.
    InvalidAssetPath {
        /// Invalid asset path.
        path: String,
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// Asset manifest contained the same GUID more than once.
    DuplicateAssetGuid {
        /// Duplicate stable asset identity.
        guid: AssetGuid,
    },
    /// Asset manifest contained the same project path more than once.
    DuplicateAssetPath {
        /// Duplicate project asset path.
        path: AssetPath,
    },
    /// Importer metadata was empty or malformed.
    InvalidImporterMetadata {
        /// Metadata field name.
        field: &'static str,
        /// Invalid field value.
        value: String,
    },
    /// Manifest entry source asset was not present in observed project state.
    MissingSourceAsset {
        /// Missing stable asset identity.
        guid: AssetGuid,
        /// Missing project asset path.
        path: AssetPath,
    },
    /// Asset reference pointed at a GUID that is absent from the manifest.
    MissingAssetReference {
        /// Referenced stable asset identity.
        guid: AssetGuid,
        /// Stored readable asset path.
        path: AssetPath,
    },
    /// Asset reference path no longer matches the manifest path for its GUID.
    AssetReferencePathMismatch {
        /// Referenced stable asset identity.
        guid: AssetGuid,
        /// Path stored on the reference.
        stored_path: AssetPath,
        /// Current manifest path for the same GUID.
        manifest_path: AssetPath,
    },
    /// Project layout validation found missing required paths.
    MissingProjectPaths {
        /// Missing workspace-relative paths.
        paths: Vec<String>,
    },
}

impl ResourceError {
    /// Stable diagnostic code for invalid asset GUID values.
    pub const INVALID_ASSET_GUID_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_INVALID_ASSET_GUID");

    /// Stable diagnostic code for invalid asset paths.
    pub const INVALID_ASSET_PATH_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_INVALID_ASSET_PATH");

    /// Stable diagnostic code for duplicate asset manifest entries.
    pub const DUPLICATE_ASSET_ENTRY_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_DUPLICATE_ASSET_ENTRY");

    /// Stable diagnostic code for invalid importer metadata.
    pub const INVALID_IMPORTER_METADATA_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_INVALID_IMPORTER_METADATA");

    /// Stable diagnostic code for missing source assets.
    pub const MISSING_SOURCE_ASSET_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_MISSING_SOURCE_ASSET");

    /// Stable diagnostic code for asset references whose GUID is not in the manifest.
    pub const MISSING_ASSET_REFERENCE_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_MISSING_ASSET_REFERENCE");

    /// Stable diagnostic code for asset references with stale readable paths.
    pub const ASSET_REFERENCE_PATH_MISMATCH_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_ASSET_REFERENCE_PATH_MISMATCH");

    /// Stable diagnostic code for missing project layout paths.
    pub const MISSING_PROJECT_PATHS_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_MISSING_PROJECT_PATHS");

    /// Diagnostic source for resource-owned validation.
    pub const RESOURCE_SOURCE: DiagnosticSource = DiagnosticSource::new("Resource");

    /// Returns the stable diagnostic code for this resource error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::InvalidAssetGuid { .. } => Self::INVALID_ASSET_GUID_CODE,
            Self::EmptyAssetPath | Self::InvalidAssetPath { .. } => Self::INVALID_ASSET_PATH_CODE,
            Self::DuplicateAssetGuid { .. } | Self::DuplicateAssetPath { .. } => {
                Self::DUPLICATE_ASSET_ENTRY_CODE
            }
            Self::InvalidImporterMetadata { .. } => Self::INVALID_IMPORTER_METADATA_CODE,
            Self::MissingSourceAsset { .. } => Self::MISSING_SOURCE_ASSET_CODE,
            Self::MissingAssetReference { .. } => Self::MISSING_ASSET_REFERENCE_CODE,
            Self::AssetReferencePathMismatch { .. } => Self::ASSET_REFERENCE_PATH_MISMATCH_CODE,
            Self::MissingProjectPaths { .. } => Self::MISSING_PROJECT_PATHS_CODE,
        }
    }

    /// Converts this error into a structured diagnostic.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let mut location = DiagnosticLocation::new();
        match self {
            Self::InvalidAssetPath { path, .. } => location.asset_path = Some(path.clone()),
            Self::DuplicateAssetPath { path }
            | Self::MissingSourceAsset { path, .. }
            | Self::MissingAssetReference { path, .. } => {
                location.asset_path = Some(path.as_str().to_owned());
            }
            Self::AssetReferencePathMismatch { stored_path, .. } => {
                location.asset_path = Some(stored_path.as_str().to_owned());
            }
            _ => {}
        }
        let blocking = match self {
            Self::MissingProjectPaths { .. } => DiagnosticBlockingScope::Build,
            Self::MissingAssetReference { .. } | Self::AssetReferencePathMismatch { .. } => {
                DiagnosticBlockingScope::Save
            }
            _ => DiagnosticBlockingScope::Import,
        };
        Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            Self::RESOURCE_SOURCE,
            self.to_string(),
        )
        .with_blocking_scope(blocking)
        .with_location(location)
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAssetGuid { raw } => {
                write!(f, "asset GUID raw value must be non-zero: {raw}")
            }
            Self::EmptyAssetPath => f.write_str("asset path must not be empty"),
            Self::InvalidAssetPath { path, reason } => {
                write!(f, "invalid asset path {path}: {reason}")
            }
            Self::DuplicateAssetGuid { guid } => {
                write!(f, "asset manifest contains duplicate GUID: {guid}")
            }
            Self::DuplicateAssetPath { path } => {
                write!(f, "asset manifest contains duplicate path: {path}")
            }
            Self::InvalidImporterMetadata { field, value } => {
                write!(f, "asset manifest importer {field} is invalid: {value}")
            }
            Self::MissingSourceAsset { guid, path } => {
                write!(f, "source asset is missing for {guid} at {path}")
            }
            Self::MissingAssetReference { guid, path } => {
                write!(f, "asset reference points to missing {guid} at {path}")
            }
            Self::AssetReferencePathMismatch {
                guid,
                stored_path,
                manifest_path,
            } => write!(
                f,
                "asset reference path for {guid} is stale: stored {stored_path}, manifest {manifest_path}"
            ),
            Self::MissingProjectPaths { paths } => {
                write!(f, "project layout is missing required paths: ")?;
                for (index, path) in paths.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_str(path)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ResourceError {}
