use core::fmt;
use std::collections::BTreeSet;

use crate::{AssetGuid, AssetPath, AssetReference, ResourceError, ResourceResult};

/// Asset import settings hash placeholder used by the in-memory manifest model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImportSettingsHash(String);

impl ImportSettingsHash {
    /// Creates an import settings hash after validation.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidImporterMetadata`] when the hash is empty.
    pub fn new(hash: impl Into<String>) -> ResourceResult<Self> {
        let hash = hash.into();
        validate_importer_field("settings_hash", &hash)?;
        Ok(Self(hash))
    }

    /// Returns the hash string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ImportSettingsHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// In-memory asset manifest entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetManifestEntry {
    reference: AssetReference,
    importer_id: String,
    importer_version: String,
    settings_hash: ImportSettingsHash,
}

impl AssetManifestEntry {
    /// Creates a manifest entry from validated asset identity and importer metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when importer metadata is empty.
    pub fn new(
        reference: AssetReference,
        importer_id: impl Into<String>,
        importer_version: impl Into<String>,
        settings_hash: ImportSettingsHash,
    ) -> ResourceResult<Self> {
        let importer_id = importer_id.into();
        let importer_version = importer_version.into();
        validate_importer_field("id", &importer_id)?;
        validate_importer_field("version", &importer_version)?;
        Ok(Self {
            reference,
            importer_id,
            importer_version,
            settings_hash,
        })
    }

    /// Creates a manifest entry from raw path and importer fields.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when the path or importer metadata is invalid.
    pub fn from_parts(
        guid: AssetGuid,
        path: impl Into<String>,
        importer_id: impl Into<String>,
        importer_version: impl Into<String>,
        settings_hash: impl Into<String>,
    ) -> ResourceResult<Self> {
        Self::new(
            AssetReference::new(guid, AssetPath::new(path)?),
            importer_id,
            importer_version,
            ImportSettingsHash::new(settings_hash)?,
        )
    }

    /// Returns the asset reference.
    #[must_use]
    pub const fn reference(&self) -> &AssetReference {
        &self.reference
    }

    /// Returns stable asset identity.
    #[must_use]
    pub const fn guid(&self) -> AssetGuid {
        self.reference.guid()
    }

    /// Returns the readable project asset path.
    #[must_use]
    pub const fn path(&self) -> &AssetPath {
        self.reference.path()
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

    /// Converts this in-memory entry into a dependency-free document contract.
    #[must_use]
    pub fn to_document(&self) -> AssetManifestEntryDocument {
        AssetManifestEntryDocument {
            guid: self.guid().raw(),
            path: self.path().as_str().to_owned(),
            importer_id: self.importer_id.clone(),
            importer_version: self.importer_version.clone(),
            settings_hash: self.settings_hash.as_str().to_owned(),
        }
    }
}

/// Deterministic in-memory asset manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssetManifest {
    entries: Vec<AssetManifestEntry>,
}

impl AssetManifest {
    /// Creates an empty asset manifest.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Creates a manifest from entries, validating uniqueness and ordering by path.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when entries contain duplicate GUIDs or paths.
    pub fn from_entries(entries: Vec<AssetManifestEntry>) -> ResourceResult<Self> {
        let mut manifest = Self { entries };
        manifest.validate_unique_entries()?;
        manifest.sort_entries();
        Ok(manifest)
    }

    /// Inserts an entry and keeps deterministic manifest ordering.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when the entry duplicates an existing GUID or path.
    pub fn insert(&mut self, entry: AssetManifestEntry) -> ResourceResult<()> {
        if self.get_by_guid(entry.guid()).is_some() {
            return Err(ResourceError::DuplicateAssetGuid { guid: entry.guid() });
        }
        if self.get_by_path(entry.path()).is_some() {
            return Err(ResourceError::DuplicateAssetPath {
                path: entry.path().clone(),
            });
        }
        self.entries.push(entry);
        self.sort_entries();
        Ok(())
    }

    /// Returns manifest entries in deterministic path order.
    #[must_use]
    pub fn entries(&self) -> &[AssetManifestEntry] {
        &self.entries
    }

    /// Finds a manifest entry by stable asset identity.
    #[must_use]
    pub fn get_by_guid(&self, guid: AssetGuid) -> Option<&AssetManifestEntry> {
        self.entries.iter().find(|entry| entry.guid() == guid)
    }

    /// Finds a manifest entry by readable project path.
    #[must_use]
    pub fn get_by_path(&self, path: &AssetPath) -> Option<&AssetManifestEntry> {
        self.entries.iter().find(|entry| entry.path() == path)
    }

    /// Converts this in-memory manifest into a dependency-free document contract.
    #[must_use]
    pub fn to_document(&self) -> AssetManifestDocument {
        AssetManifestDocument {
            entries: self
                .entries
                .iter()
                .map(AssetManifestEntry::to_document)
                .collect(),
        }
    }

    /// Creates a validated in-memory manifest from a document contract.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when document entries contain invalid GUIDs,
    /// paths, importer metadata, or duplicate identities/paths.
    pub fn from_document(document: AssetManifestDocument) -> ResourceResult<Self> {
        let entries = document
            .entries
            .into_iter()
            .map(AssetManifestEntry::from_document)
            .collect::<ResourceResult<Vec<_>>>()?;
        Self::from_entries(entries)
    }

    fn validate_unique_entries(&self) -> ResourceResult<()> {
        let mut guids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for entry in &self.entries {
            if !guids.insert(entry.guid()) {
                return Err(ResourceError::DuplicateAssetGuid { guid: entry.guid() });
            }
            if !paths.insert(entry.path().clone()) {
                return Err(ResourceError::DuplicateAssetPath {
                    path: entry.path().clone(),
                });
            }
        }
        Ok(())
    }

    fn sort_entries(&mut self) {
        self.entries.sort_by(|left, right| {
            left.path()
                .cmp(right.path())
                .then_with(|| left.guid().cmp(&right.guid()))
        });
    }
}

/// Dependency-free `project/assets.knmanifest` document contract.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssetManifestDocument {
    /// Manifest entries. Conversion to [`AssetManifest`] validates and sorts them.
    pub entries: Vec<AssetManifestEntryDocument>,
}

impl AssetManifestDocument {
    /// Creates a document contract and normalizes entry order through manifest validation.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when entries contain invalid or duplicate data.
    pub fn new(entries: Vec<AssetManifestEntryDocument>) -> ResourceResult<Self> {
        Ok(AssetManifest::from_document(Self { entries })?.to_document())
    }
}

/// Dependency-free asset manifest entry document contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetManifestEntryDocument {
    /// Stable asset GUID raw value.
    pub guid: u64,
    /// Readable `res://` project path.
    pub path: String,
    /// Importer identifier.
    pub importer_id: String,
    /// Importer version.
    pub importer_version: String,
    /// Import settings hash.
    pub settings_hash: String,
}

impl AssetManifestEntryDocument {
    /// Creates an asset manifest entry document.
    #[must_use]
    pub fn new(
        guid: u64,
        path: impl Into<String>,
        importer_id: impl Into<String>,
        importer_version: impl Into<String>,
        settings_hash: impl Into<String>,
    ) -> Self {
        Self {
            guid,
            path: path.into(),
            importer_id: importer_id.into(),
            importer_version: importer_version.into(),
            settings_hash: settings_hash.into(),
        }
    }
}

impl AssetManifestEntry {
    fn from_document(document: AssetManifestEntryDocument) -> ResourceResult<Self> {
        Self::from_parts(
            AssetGuid::try_new(document.guid)?,
            document.path,
            document.importer_id,
            document.importer_version,
            document.settings_hash,
        )
    }
}

fn validate_importer_field(field: &'static str, value: &str) -> ResourceResult<()> {
    if value.trim().is_empty() {
        return Err(ResourceError::InvalidImporterMetadata {
            field,
            value: value.to_owned(),
        });
    }
    if value.trim() != value {
        return Err(ResourceError::InvalidImporterMetadata {
            field,
            value: value.to_owned(),
        });
    }
    Ok(())
}
