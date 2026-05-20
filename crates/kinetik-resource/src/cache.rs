use core::{fmt, num::NonZeroU32};

use crate::{AssetGuid, ImportSettingsHash, ResourceError, ResourceResult};

/// Source asset content hash used by import cache records.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceContentHash(String);

impl SourceContentHash {
    /// Creates a source content hash after validation.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidImporterMetadata`] when the hash is empty.
    pub fn new(hash: impl Into<String>) -> ResourceResult<Self> {
        let hash = hash.into();
        validate_cache_text_field("source_content_hash", &hash)?;
        Ok(Self(hash))
    }

    /// Returns the hash string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Engine import cache schema version.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImportCacheSchemaVersion(NonZeroU32);

impl ImportCacheSchemaVersion {
    /// Creates an import cache schema version from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        match NonZeroU32::new(raw) {
            Some(raw) => Self(raw),
            None => panic!("ImportCacheSchemaVersion raw value must be non-zero"),
        }
    }

    /// Creates an import cache schema version from a raw value.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidImporterMetadata`] when `raw` is zero.
    pub fn try_new(raw: u32) -> ResourceResult<Self> {
        match NonZeroU32::new(raw) {
            Some(raw) => Ok(Self(raw)),
            None => Err(ResourceError::InvalidImporterMetadata {
                field: "cache_schema_version",
                value: raw.to_string(),
            }),
        }
    }

    /// Returns the raw schema version.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for ImportCacheSchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw())
    }
}

/// Dependency-free import cache metadata for one imported source asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportCacheRecord {
    asset_guid: AssetGuid,
    source_content_hash: SourceContentHash,
    importer_id: String,
    importer_version: String,
    settings_hash: ImportSettingsHash,
    cache_schema_version: ImportCacheSchemaVersion,
}

impl ImportCacheRecord {
    /// Creates import cache metadata after validating required fields.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidImporterMetadata`] when metadata fields are empty.
    pub fn new(
        asset_guid: AssetGuid,
        source_content_hash: SourceContentHash,
        importer_id: impl Into<String>,
        importer_version: impl Into<String>,
        settings_hash: ImportSettingsHash,
        cache_schema_version: ImportCacheSchemaVersion,
    ) -> ResourceResult<Self> {
        let importer_id = importer_id.into();
        let importer_version = importer_version.into();
        validate_cache_text_field("id", &importer_id)?;
        validate_cache_text_field("version", &importer_version)?;
        Ok(Self {
            asset_guid,
            source_content_hash,
            importer_id,
            importer_version,
            settings_hash,
            cache_schema_version,
        })
    }

    /// Returns the stable asset identity.
    #[must_use]
    pub const fn asset_guid(&self) -> AssetGuid {
        self.asset_guid
    }

    /// Returns the source content hash.
    #[must_use]
    pub const fn source_content_hash(&self) -> &SourceContentHash {
        &self.source_content_hash
    }

    /// Returns the importer identifier.
    #[must_use]
    pub fn importer_id(&self) -> &str {
        &self.importer_id
    }

    /// Returns the importer version.
    #[must_use]
    pub fn importer_version(&self) -> &str {
        &self.importer_version
    }

    /// Returns the import settings hash.
    #[must_use]
    pub const fn settings_hash(&self) -> &ImportSettingsHash {
        &self.settings_hash
    }

    /// Returns the engine import cache schema version.
    #[must_use]
    pub const fn cache_schema_version(&self) -> ImportCacheSchemaVersion {
        self.cache_schema_version
    }
}

fn validate_cache_text_field(field: &'static str, value: &str) -> ResourceResult<()> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(ResourceError::InvalidImporterMetadata {
            field,
            value: value.to_owned(),
        });
    }
    Ok(())
}
