//! Resource handles and asset path contracts for Kinetik.

use core::{fmt, num::NonZeroU64};
use std::collections::BTreeSet;

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
            Self::MissingProjectPaths { .. } => Self::MISSING_PROJECT_PATHS_CODE,
        }
    }

    /// Converts this error into a structured diagnostic.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let mut location = DiagnosticLocation::new();
        match self {
            Self::InvalidAssetPath { path, .. } => location.asset_path = Some(path.clone()),
            Self::DuplicateAssetPath { path } => {
                location.asset_path = Some(path.as_str().to_owned());
            }
            _ => {}
        }
        let blocking = match self {
            Self::MissingProjectPaths { .. } => DiagnosticBlockingScope::Build,
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

/// Dependency-free `project/assets.ktmanifest` document contract.
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

/// Logical purpose of a canonical project workspace path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectPathKind {
    /// Project settings file.
    ProjectSettings,
    /// Scene source directory.
    ScenesDirectory,
    /// Default scene source file.
    MainScene,
    /// Prefab source directory.
    PrefabsDirectory,
    /// Script source directory.
    ScriptsDirectory,
    /// Source asset directory.
    AssetsDirectory,
    /// Stable metadata directory.
    ProjectMetadataDirectory,
    /// Asset manifest file.
    AssetsManifest,
    /// Instance manifest file.
    InstancesManifest,
    /// Generated Kinetik directory.
    KinetikDirectory,
    /// Generated cache directory.
    CacheDirectory,
    /// Generated import-output directory.
    ImportDirectory,
    /// Generated build-output directory.
    BuildDirectory,
}

/// Whether a project layout path is committed source or disposable generated output.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectPathDomain {
    /// Source-controlled project path.
    Source,
    /// Generated disposable path under `.kinetik/`.
    Generated,
}

/// Canonical workspace-relative project path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectLayoutPath {
    /// Logical path kind.
    pub kind: ProjectPathKind,
    /// Workspace-relative path using `/` separators.
    pub path: &'static str,
    /// Source or generated path domain.
    pub domain: ProjectPathDomain,
}

impl ProjectLayoutPath {
    /// Returns whether this path is required for source project validation.
    #[must_use]
    pub const fn is_required_source(self) -> bool {
        matches!(self.domain, ProjectPathDomain::Source)
    }
}

/// In-memory model of the default Kinetik project workspace layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLayout {
    paths: Vec<ProjectLayoutPath>,
}

impl ProjectLayout {
    /// Creates the default Kinetik project layout model.
    #[must_use]
    pub fn new() -> Self {
        Self {
            paths: DEFAULT_PROJECT_LAYOUT_PATHS.to_vec(),
        }
    }

    /// Returns all scaffold paths in deterministic order.
    #[must_use]
    pub fn scaffold_paths(&self) -> &[ProjectLayoutPath] {
        &self.paths
    }

    /// Returns source-controlled paths required for project validation.
    #[must_use]
    pub fn required_source_paths(&self) -> Vec<ProjectLayoutPath> {
        self.paths
            .iter()
            .copied()
            .filter(|path| path.is_required_source())
            .collect()
    }

    /// Returns a canonical path by logical kind.
    #[must_use]
    pub fn path(&self, kind: ProjectPathKind) -> Option<&'static str> {
        self.paths
            .iter()
            .find(|path| path.kind == kind)
            .map(|path| path.path)
    }

    /// Validates that a caller-provided path set contains every required source path.
    ///
    /// Generated `.kinetik/` paths are intentionally excluded because they are
    /// disposable and can be rebuilt.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::MissingProjectPaths`] with all missing paths in
    /// deterministic layout order.
    pub fn validate_required_paths<I, P>(&self, paths: I) -> ResourceResult<()>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        let present_paths: BTreeSet<String> = paths
            .into_iter()
            .map(|path| normalize_project_path(path.as_ref()))
            .collect();
        let missing_paths: Vec<String> = self
            .required_source_paths()
            .into_iter()
            .map(|path| path.path)
            .filter(|path| !present_paths.contains(*path))
            .map(str::to_owned)
            .collect();

        if missing_paths.is_empty() {
            Ok(())
        } else {
            Err(ResourceError::MissingProjectPaths {
                paths: missing_paths,
            })
        }
    }
}

impl Default for ProjectLayout {
    fn default() -> Self {
        Self::new()
    }
}

const DEFAULT_PROJECT_LAYOUT_PATHS: [ProjectLayoutPath; 13] = [
    ProjectLayoutPath {
        kind: ProjectPathKind::ProjectSettings,
        path: "Kinetik.toml",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ScenesDirectory,
        path: "scenes",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::MainScene,
        path: "scenes/main.ktscene",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::PrefabsDirectory,
        path: "prefabs",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ScriptsDirectory,
        path: "scripts",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::AssetsDirectory,
        path: "assets",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ProjectMetadataDirectory,
        path: "project",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::AssetsManifest,
        path: "project/assets.ktmanifest",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::InstancesManifest,
        path: "project/instances.ktmanifest",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::KinetikDirectory,
        path: ".kinetik",
        domain: ProjectPathDomain::Generated,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::CacheDirectory,
        path: ".kinetik/cache",
        domain: ProjectPathDomain::Generated,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ImportDirectory,
        path: ".kinetik/import",
        domain: ProjectPathDomain::Generated,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::BuildDirectory,
        path: ".kinetik/build",
        domain: ProjectPathDomain::Generated,
    },
];

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

fn normalize_project_path(path: &str) -> String {
    path.trim_end_matches('/').replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_entry(
        guid: u64,
        path: &str,
        importer_id: &str,
    ) -> ResourceResult<AssetManifestEntry> {
        AssetManifestEntry::from_parts(
            AssetGuid::new(guid),
            path,
            importer_id,
            "1.0.0",
            "settings-hash",
        )
    }

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

    #[test]
    fn manifest_entries_store_identity_path_and_importer_metadata() {
        let entry = manifest_entry(1, "res://assets/models/tree.glb", "gltf").unwrap();

        assert_eq!(entry.guid(), AssetGuid::new(1));
        assert_eq!(entry.path().as_str(), "res://assets/models/tree.glb");
        assert_eq!(entry.importer_id(), "gltf");
        assert_eq!(entry.importer_version(), "1.0.0");
        assert_eq!(entry.settings_hash().as_str(), "settings-hash");
    }

    #[test]
    fn manifest_entries_reject_missing_paths_and_invalid_importer_metadata() {
        assert_eq!(
            AssetManifestEntry::from_parts(AssetGuid::new(1), "", "gltf", "1.0.0", "hash")
                .unwrap_err(),
            ResourceError::EmptyAssetPath
        );
        assert_eq!(
            AssetManifestEntry::from_parts(
                AssetGuid::new(1),
                "res://assets/tree.glb",
                " ",
                "1.0.0",
                "hash"
            )
            .unwrap_err(),
            ResourceError::InvalidImporterMetadata {
                field: "id",
                value: " ".to_owned()
            }
        );
        assert_eq!(
            ImportSettingsHash::new(" hash ").unwrap_err(),
            ResourceError::InvalidImporterMetadata {
                field: "settings_hash",
                value: " hash ".to_owned()
            }
        );
    }

    #[test]
    fn manifest_orders_entries_deterministically_by_path() {
        let manifest = AssetManifest::from_entries(vec![
            manifest_entry(2, "res://assets/models/tree.glb", "gltf").unwrap(),
            manifest_entry(1, "res://assets/audio/theme.ogg", "audio").unwrap(),
            manifest_entry(3, "res://assets/materials/bark.ktmat", "material").unwrap(),
        ])
        .unwrap();

        let ordered_paths: Vec<&str> = manifest
            .entries()
            .iter()
            .map(|entry| entry.path().as_str())
            .collect();
        assert_eq!(
            ordered_paths,
            vec![
                "res://assets/audio/theme.ogg",
                "res://assets/materials/bark.ktmat",
                "res://assets/models/tree.glb"
            ]
        );
    }

    #[test]
    fn manifest_rejects_duplicate_guids_and_paths() {
        assert_eq!(
            AssetManifest::from_entries(vec![
                manifest_entry(1, "res://assets/a.glb", "gltf").unwrap(),
                manifest_entry(1, "res://assets/b.glb", "gltf").unwrap(),
            ])
            .unwrap_err(),
            ResourceError::DuplicateAssetGuid {
                guid: AssetGuid::new(1)
            }
        );

        assert_eq!(
            AssetManifest::from_entries(vec![
                manifest_entry(1, "res://assets/a.glb", "gltf").unwrap(),
                manifest_entry(2, "res://assets/a.glb", "gltf").unwrap(),
            ])
            .unwrap_err(),
            ResourceError::DuplicateAssetPath {
                path: AssetPath::new("res://assets/a.glb").unwrap()
            }
        );
    }

    #[test]
    fn manifest_insert_validates_duplicates_and_keeps_order() {
        let mut manifest = AssetManifest::new();
        manifest
            .insert(manifest_entry(2, "res://assets/z.glb", "gltf").unwrap())
            .unwrap();
        manifest
            .insert(manifest_entry(1, "res://assets/a.glb", "gltf").unwrap())
            .unwrap();

        assert_eq!(manifest.entries()[0].guid(), AssetGuid::new(1));
        assert_eq!(
            manifest
                .insert(manifest_entry(1, "res://assets/b.glb", "gltf").unwrap())
                .unwrap_err(),
            ResourceError::DuplicateAssetGuid {
                guid: AssetGuid::new(1)
            }
        );
    }

    #[test]
    fn manifest_looks_up_entries_by_guid_and_path() {
        let manifest = AssetManifest::from_entries(vec![
            manifest_entry(1, "res://assets/a.glb", "gltf").unwrap(),
            manifest_entry(2, "res://assets/b.glb", "gltf").unwrap(),
        ])
        .unwrap();

        assert_eq!(
            manifest
                .get_by_guid(AssetGuid::new(2))
                .unwrap()
                .path()
                .as_str(),
            "res://assets/b.glb"
        );
        assert_eq!(
            manifest
                .get_by_path(&AssetPath::new("res://assets/a.glb").unwrap())
                .unwrap()
                .guid(),
            AssetGuid::new(1)
        );
        assert!(manifest.get_by_guid(AssetGuid::new(99)).is_none());
    }

    #[test]
    fn asset_manifest_document_converts_from_manifest_in_deterministic_order() {
        let manifest = AssetManifest::from_entries(vec![
            manifest_entry(2, "res://assets/z.glb", "gltf").unwrap(),
            manifest_entry(1, "res://assets/a.glb", "gltf").unwrap(),
        ])
        .unwrap();

        let document = manifest.to_document();

        assert_eq!(
            document.entries,
            vec![
                AssetManifestEntryDocument::new(
                    1,
                    "res://assets/a.glb",
                    "gltf",
                    "1.0.0",
                    "settings-hash"
                ),
                AssetManifestEntryDocument::new(
                    2,
                    "res://assets/z.glb",
                    "gltf",
                    "1.0.0",
                    "settings-hash"
                )
            ]
        );
    }

    #[test]
    fn asset_manifest_document_round_trips_through_validated_manifest() {
        let document = AssetManifestDocument::new(vec![
            AssetManifestEntryDocument::new(
                2,
                "res://assets/models/tree.glb",
                "gltf",
                "1.0.0",
                "hash-b",
            ),
            AssetManifestEntryDocument::new(
                1,
                "res://assets/audio/theme.ogg",
                "audio",
                "1.0.0",
                "hash-a",
            ),
        ])
        .unwrap();

        let manifest = AssetManifest::from_document(document.clone()).unwrap();

        assert_eq!(manifest.to_document(), document);
        assert_eq!(
            manifest.entries()[0].path().as_str(),
            "res://assets/audio/theme.ogg"
        );
        assert_eq!(
            manifest.entries()[1].path().as_str(),
            "res://assets/models/tree.glb"
        );
    }

    #[test]
    fn asset_manifest_document_rejects_invalid_fields() {
        assert_eq!(
            AssetManifest::from_document(AssetManifestDocument {
                entries: vec![AssetManifestEntryDocument::new(
                    0,
                    "res://assets/a.glb",
                    "gltf",
                    "1.0.0",
                    "hash"
                )]
            })
            .unwrap_err(),
            ResourceError::InvalidAssetGuid { raw: 0 }
        );

        assert_eq!(
            AssetManifest::from_document(AssetManifestDocument {
                entries: vec![AssetManifestEntryDocument::new(
                    1, "", "gltf", "1.0.0", "hash"
                )]
            })
            .unwrap_err(),
            ResourceError::EmptyAssetPath
        );

        assert_eq!(
            AssetManifest::from_document(AssetManifestDocument {
                entries: vec![AssetManifestEntryDocument::new(
                    1,
                    "res://assets/a.glb",
                    "",
                    "1.0.0",
                    "hash"
                )]
            })
            .unwrap_err(),
            ResourceError::InvalidImporterMetadata {
                field: "id",
                value: String::new()
            }
        );
    }

    #[test]
    fn asset_manifest_document_rejects_duplicate_identities_and_paths() {
        assert_eq!(
            AssetManifestDocument::new(vec![
                AssetManifestEntryDocument::new(1, "res://assets/a.glb", "gltf", "1.0.0", "hash"),
                AssetManifestEntryDocument::new(1, "res://assets/b.glb", "gltf", "1.0.0", "hash")
            ])
            .unwrap_err(),
            ResourceError::DuplicateAssetGuid {
                guid: AssetGuid::new(1)
            }
        );

        assert_eq!(
            AssetManifestDocument::new(vec![
                AssetManifestEntryDocument::new(1, "res://assets/a.glb", "gltf", "1.0.0", "hash"),
                AssetManifestEntryDocument::new(2, "res://assets/a.glb", "gltf", "1.0.0", "hash")
            ])
            .unwrap_err(),
            ResourceError::DuplicateAssetPath {
                path: AssetPath::new("res://assets/a.glb").unwrap()
            }
        );
    }

    #[test]
    fn manifest_errors_include_diagnostic_context_when_available() {
        let error = ResourceError::DuplicateAssetPath {
            path: AssetPath::new("res://assets/a.glb").unwrap(),
        };
        let diagnostic = error.to_diagnostic();

        assert_eq!(diagnostic.code, ResourceError::DUPLICATE_ASSET_ENTRY_CODE);
        assert_eq!(
            diagnostic.location.asset_path.as_deref(),
            Some("res://assets/a.glb")
        );
    }

    #[test]
    fn project_layout_lists_default_scaffold_paths_in_order() {
        let layout = ProjectLayout::new();
        let paths: Vec<&str> = layout
            .scaffold_paths()
            .iter()
            .map(|path| path.path)
            .collect();

        assert_eq!(
            paths,
            vec![
                "Kinetik.toml",
                "scenes",
                "scenes/main.ktscene",
                "prefabs",
                "scripts",
                "assets",
                "project",
                "project/assets.ktmanifest",
                "project/instances.ktmanifest",
                ".kinetik",
                ".kinetik/cache",
                ".kinetik/import",
                ".kinetik/build"
            ]
        );
        assert_eq!(
            layout.path(ProjectPathKind::AssetsManifest),
            Some("project/assets.ktmanifest")
        );
    }

    #[test]
    fn project_layout_separates_required_source_from_generated_paths() {
        let layout = ProjectLayout::new();
        let required_paths: Vec<&str> = layout
            .required_source_paths()
            .iter()
            .map(|path| path.path)
            .collect();

        assert_eq!(
            required_paths,
            vec![
                "Kinetik.toml",
                "scenes",
                "scenes/main.ktscene",
                "prefabs",
                "scripts",
                "assets",
                "project",
                "project/assets.ktmanifest",
                "project/instances.ktmanifest"
            ]
        );
        assert!(
            layout
                .scaffold_paths()
                .iter()
                .find(|path| path.path == ".kinetik/cache")
                .unwrap()
                .domain
                == ProjectPathDomain::Generated
        );
    }

    #[test]
    fn project_layout_validates_required_paths_without_requiring_generated_output() {
        let layout = ProjectLayout::new();
        let present_paths = [
            "Kinetik.toml",
            "scenes/",
            "scenes/main.ktscene",
            "prefabs",
            "scripts",
            "assets",
            "project",
            "project/assets.ktmanifest",
            "project/instances.ktmanifest",
        ];

        layout.validate_required_paths(present_paths).unwrap();
    }

    #[test]
    fn project_layout_reports_missing_paths_in_deterministic_order() {
        let layout = ProjectLayout::new();
        let error = layout
            .validate_required_paths(["Kinetik.toml", "assets", ".kinetik/cache"])
            .unwrap_err();

        assert_eq!(
            error,
            ResourceError::MissingProjectPaths {
                paths: vec![
                    "scenes".to_owned(),
                    "scenes/main.ktscene".to_owned(),
                    "prefabs".to_owned(),
                    "scripts".to_owned(),
                    "project".to_owned(),
                    "project/assets.ktmanifest".to_owned(),
                    "project/instances.ktmanifest".to_owned()
                ]
            }
        );

        let diagnostic = error.to_diagnostic();
        assert_eq!(diagnostic.code, ResourceError::MISSING_PROJECT_PATHS_CODE);
        assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Build));
    }
}
