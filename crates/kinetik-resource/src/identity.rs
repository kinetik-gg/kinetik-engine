use core::{fmt, num::NonZeroU64};

use kinetik_core::ResourceId;

use crate::{ResourceError, ResourceResult};

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
