use std::collections::BTreeSet;

use kinetik_core::Diagnostic;

use crate::ResourceError;
use crate::{AssetGuid, AssetManifest, AssetManifestEntry, AssetPath, ResourceResult};

/// Engine-owned resource database over committed project manifests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceDatabase {
    manifest: AssetManifest,
}

impl ResourceDatabase {
    /// Creates an empty resource database.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            manifest: AssetManifest::new(),
        }
    }

    /// Creates a resource database from a validated asset manifest.
    #[must_use]
    pub const fn from_manifest(manifest: AssetManifest) -> Self {
        Self { manifest }
    }

    /// Returns the committed asset manifest backing this database.
    #[must_use]
    pub const fn manifest(&self) -> &AssetManifest {
        &self.manifest
    }

    /// Returns database entries in deterministic project path order.
    #[must_use]
    pub fn entries(&self) -> &[AssetManifestEntry] {
        self.manifest.entries()
    }

    /// Looks up a database entry by stable asset GUID.
    #[must_use]
    pub fn get_by_guid(&self, guid: AssetGuid) -> Option<&AssetManifestEntry> {
        self.manifest.get_by_guid(guid)
    }

    /// Looks up a database entry by validated `res://` project path.
    #[must_use]
    pub fn get_by_path(&self, path: &AssetPath) -> Option<&AssetManifestEntry> {
        self.manifest.get_by_path(path)
    }

    /// Validates and looks up a database entry by raw `res://` project path.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ResourceError`] when the raw path does not follow the
    /// `res://` project path contract.
    pub fn get_by_res_path(
        &self,
        path: impl Into<String>,
    ) -> ResourceResult<Option<&AssetManifestEntry>> {
        let path = AssetPath::new(path)?;
        Ok(self.get_by_path(&path))
    }

    /// Reports manifest entries whose source paths are not present in observed project state.
    ///
    /// The caller supplies source paths observed by a higher-level workspace or
    /// file-system layer. This keeps the database deterministic and free of IO
    /// dependencies.
    #[must_use]
    pub fn missing_source_diagnostics<I>(&self, observed_paths: I) -> Vec<Diagnostic>
    where
        I: IntoIterator<Item = AssetPath>,
    {
        let observed_paths = observed_paths.into_iter().collect::<BTreeSet<_>>();
        self.entries()
            .iter()
            .filter(|entry| !observed_paths.contains(entry.path()))
            .map(|entry| {
                ResourceError::MissingSourceAsset {
                    guid: entry.guid(),
                    path: entry.path().clone(),
                }
                .to_diagnostic()
            })
            .collect()
    }
}
