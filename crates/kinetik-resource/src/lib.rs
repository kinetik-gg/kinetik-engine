//! Resource handles and asset path contracts for Kinetik.

use core::{fmt, num::NonZeroU64};

use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticLocation, DiagnosticSeverity,
    DiagnosticSource, ResourceId,
};

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
}

impl ResourceError {
    /// Stable diagnostic code for invalid asset GUID values.
    pub const INVALID_ASSET_GUID_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_INVALID_ASSET_GUID");

    /// Stable diagnostic code for invalid asset paths.
    pub const INVALID_ASSET_PATH_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_RESOURCE_INVALID_ASSET_PATH");

    /// Diagnostic source for resource-owned validation.
    pub const RESOURCE_SOURCE: DiagnosticSource = DiagnosticSource::new("Resource");

    /// Returns the stable diagnostic code for this resource error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::InvalidAssetGuid { .. } => Self::INVALID_ASSET_GUID_CODE,
            Self::EmptyAssetPath | Self::InvalidAssetPath { .. } => Self::INVALID_ASSET_PATH_CODE,
        }
    }

    /// Converts this error into a structured diagnostic.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let mut location = DiagnosticLocation::new();
        if let Self::InvalidAssetPath { path, .. } = self {
            location.asset_path = Some(path.clone());
        }
        Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            Self::RESOURCE_SOURCE,
            self.to_string(),
        )
        .with_blocking_scope(DiagnosticBlockingScope::Import)
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
        }
    }
}

impl std::error::Error for ResourceError {}

/// Stable source asset identity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetGuid(NonZeroU64);

impl AssetGuid {
    /// Creates a stable asset GUID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        match NonZeroU64::new(raw) {
            Some(raw) => Self(raw),
            None => panic!("AssetGuid raw value must be non-zero"),
        }
    }

    /// Creates a stable asset GUID from a raw value.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidAssetGuid`] when `raw` is zero.
    pub const fn try_new(raw: u64) -> ResourceResult<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Ok(Self(raw)),
            None => Err(ResourceError::InvalidAssetGuid { raw }),
        }
    }

    /// Returns the raw numeric value for serialization/debugging.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for AssetGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetGuid({})", self.raw())
    }
}

/// Logical project asset path, such as `res://assets/models/tree.glb`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetPath(String);

impl AssetPath {
    /// Required project asset path scheme.
    pub const SCHEME: &'static str = "res://";

    /// Creates an asset path after validating the `res://` project path contract.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when the path is empty or malformed.
    pub fn new(path: impl Into<String>) -> ResourceResult<Self> {
        let path = path.into();
        validate_asset_path(&path)?;
        Ok(Self(path))
    }

    /// Returns the asset path as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a new asset path with the same validation rules.
    ///
    /// This is useful when a manifest updates a moved or renamed source asset
    /// while preserving stable identity elsewhere.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when `path` is empty or malformed.
    pub fn moved_to(&self, path: impl Into<String>) -> ResourceResult<Self> {
        Self::new(path)
    }
}

impl fmt::Display for AssetPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable source asset reference with both durable identity and readable path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetReference {
    guid: AssetGuid,
    path: AssetPath,
}

impl AssetReference {
    /// Creates an asset reference from stable identity and readable path.
    #[must_use]
    pub const fn new(guid: AssetGuid, path: AssetPath) -> Self {
        Self { guid, path }
    }

    /// Returns stable asset identity.
    #[must_use]
    pub const fn guid(&self) -> AssetGuid {
        self.guid
    }

    /// Returns the readable project asset path.
    #[must_use]
    pub const fn path(&self) -> &AssetPath {
        &self.path
    }

    /// Returns a reference with the same stable identity and a new path.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when `path` is empty or malformed.
    pub fn with_path(&self, path: impl Into<String>) -> ResourceResult<Self> {
        Ok(Self {
            guid: self.guid,
            path: self.path.moved_to(path)?,
        })
    }
}

/// Typed resource handle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceHandle {
    id: ResourceId,
}

impl ResourceHandle {
    /// Creates a new resource handle.
    #[must_use]
    pub const fn new(id: ResourceId) -> Self {
        Self { id }
    }

    /// Returns the underlying resource ID.
    #[must_use]
    pub const fn id(self) -> ResourceId {
        self.id
    }
}

fn validate_asset_path(path: &str) -> ResourceResult<()> {
    if path.is_empty() {
        return Err(ResourceError::EmptyAssetPath);
    }
    if path.trim() != path {
        return Err(invalid_asset_path(
            path,
            "must not contain leading or trailing whitespace",
        ));
    }
    let Some(relative_path) = path.strip_prefix(AssetPath::SCHEME) else {
        return Err(invalid_asset_path(path, "must start with res://"));
    };
    if relative_path.is_empty() {
        return Err(invalid_asset_path(
            path,
            "must include a project-relative path after res://",
        ));
    }
    if relative_path.starts_with('/') {
        return Err(invalid_asset_path(
            path,
            "must not contain an absolute path after res://",
        ));
    }
    if relative_path.contains('\\') {
        return Err(invalid_asset_path(
            path,
            "must use '/' separators, not backslashes",
        ));
    }
    for segment in relative_path.split('/') {
        if segment.is_empty() {
            return Err(invalid_asset_path(
                path,
                "must not contain empty path segments",
            ));
        }
        if matches!(segment, "." | "..") {
            return Err(invalid_asset_path(
                path,
                "must not contain relative path segments",
            ));
        }
        if segment.trim() != segment {
            return Err(invalid_asset_path(
                path,
                "path segments must not contain leading or trailing whitespace",
            ));
        }
    }
    Ok(())
}

fn invalid_asset_path(path: &str, reason: &'static str) -> ResourceError {
    ResourceError::InvalidAssetPath {
        path: path.to_owned(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_guid_rejects_zero_raw_values() {
        assert_eq!(
            AssetGuid::try_new(0).unwrap_err(),
            ResourceError::InvalidAssetGuid { raw: 0 }
        );
        assert!(std::panic::catch_unwind(|| AssetGuid::new(0)).is_err());
    }

    #[test]
    fn asset_guid_display_is_stable() {
        let guid = AssetGuid::new(42);

        assert_eq!(guid.raw(), 42);
        assert_eq!(guid.to_string(), "AssetGuid(42)");
        assert_eq!(format!("{guid:?}"), "AssetGuid(42)");
    }

    #[test]
    fn asset_paths_validate_res_scheme_paths() {
        let path = AssetPath::new("res://assets/models/tree.glb").unwrap();

        assert_eq!(path.as_str(), "res://assets/models/tree.glb");
        assert_eq!(path.to_string(), "res://assets/models/tree.glb");
    }

    #[test]
    fn asset_paths_reject_empty_and_malformed_paths() {
        assert_eq!(
            AssetPath::new("").unwrap_err(),
            ResourceError::EmptyAssetPath
        );

        let invalid_cases = [
            ("assets/tree.glb", "must start with res://"),
            (
                "res://",
                "must include a project-relative path after res://",
            ),
            (
                "res:///assets/tree.glb",
                "must not contain an absolute path after res://",
            ),
            (
                "res://assets//tree.glb",
                "must not contain empty path segments",
            ),
            (
                "res://assets/../tree.glb",
                "must not contain relative path segments",
            ),
            (
                "res://assets\\tree.glb",
                "must use '/' separators, not backslashes",
            ),
            (
                " res://assets/tree.glb",
                "must not contain leading or trailing whitespace",
            ),
            (
                "res://assets/ tree.glb",
                "path segments must not contain leading or trailing whitespace",
            ),
        ];

        for (path, reason) in invalid_cases {
            assert_eq!(
                AssetPath::new(path).unwrap_err(),
                ResourceError::InvalidAssetPath {
                    path: path.to_owned(),
                    reason
                }
            );
        }
    }

    #[test]
    fn asset_references_preserve_identity_across_path_changes() {
        let reference = AssetReference::new(
            AssetGuid::new(7),
            AssetPath::new("res://assets/models/tree.glb").unwrap(),
        );
        let moved = reference
            .with_path("res://assets/environment/oak.glb")
            .unwrap();

        assert_eq!(moved.guid(), reference.guid());
        assert_eq!(moved.path().as_str(), "res://assets/environment/oak.glb");
        assert_ne!(moved.path(), reference.path());
    }

    #[test]
    fn resource_errors_convert_to_diagnostics() {
        let error = AssetPath::new("bad/path").unwrap_err();
        let diagnostic = error.to_diagnostic();

        assert_eq!(diagnostic.code, ResourceError::INVALID_ASSET_PATH_CODE);
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.source, ResourceError::RESOURCE_SOURCE);
        assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Import));
        assert_eq!(diagnostic.location.asset_path.as_deref(), Some("bad/path"));
    }
}
