use std::collections::BTreeSet;

use kinetik_core::Diagnostic;
use kinetik_reflect::{AssetReferenceValue, PropertyValue};
use kinetik_scene::Scene;

use crate::{validate_asset_kind, AssetImportKind, ResourceError};
use crate::{
    AssetGuid, AssetManifest, AssetManifestEntry, AssetPath, AssetReference, ImportCacheRecord,
    ResourceResult,
};

/// Engine-owned resource database over committed project manifests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceDatabase {
    manifest: AssetManifest,
    import_cache_records: Vec<ImportCacheRecord>,
}

impl ResourceDatabase {
    /// Creates an empty resource database.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            manifest: AssetManifest::new(),
            import_cache_records: Vec::new(),
        }
    }

    /// Creates a resource database from a validated asset manifest.
    #[must_use]
    pub const fn from_manifest(manifest: AssetManifest) -> Self {
        Self {
            manifest,
            import_cache_records: Vec::new(),
        }
    }

    /// Creates a resource database from a validated asset manifest and import cache metadata.
    ///
    /// Import cache records are sorted by asset GUID for deterministic lookup,
    /// UI, MCP, tests, and bundle preparation. Generated cache outputs remain
    /// disposable and are not loaded by this constructor.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::DuplicateImportCacheAsset`] when more than one
    /// cache record exists for the same asset GUID.
    pub fn from_manifest_and_import_cache_records(
        manifest: AssetManifest,
        mut import_cache_records: Vec<ImportCacheRecord>,
    ) -> ResourceResult<Self> {
        import_cache_records.sort_by_key(ImportCacheRecord::asset_guid);
        for pair in import_cache_records.windows(2) {
            let [previous, current] = pair else {
                unreachable!("windows(2) always yields two entries");
            };
            if previous.asset_guid() == current.asset_guid() {
                return Err(ResourceError::DuplicateImportCacheAsset {
                    guid: current.asset_guid(),
                });
            }
        }
        Ok(Self {
            manifest,
            import_cache_records,
        })
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

    /// Returns import cache metadata records in deterministic asset GUID order.
    #[must_use]
    pub fn import_cache_records(&self) -> &[ImportCacheRecord] {
        &self.import_cache_records
    }

    /// Looks up a database entry by stable asset GUID.
    #[must_use]
    pub fn get_by_guid(&self, guid: AssetGuid) -> Option<&AssetManifestEntry> {
        self.manifest.get_by_guid(guid)
    }

    /// Looks up import cache metadata by stable asset GUID.
    #[must_use]
    pub fn get_import_cache_by_guid(&self, guid: AssetGuid) -> Option<&ImportCacheRecord> {
        self.import_cache_records
            .iter()
            .find(|record| record.asset_guid() == guid)
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

    /// Validates a durable asset reference against committed manifest identity.
    ///
    /// Returns an empty vector when the reference resolves cleanly. Missing GUIDs
    /// and stale readable paths are reported as diagnostics instead of being
    /// repaired silently.
    #[must_use]
    pub fn asset_reference_diagnostics(&self, reference: &AssetReference) -> Vec<Diagnostic> {
        let Some(entry) = self.get_by_guid(reference.guid()) else {
            return vec![ResourceError::MissingAssetReference {
                guid: reference.guid(),
                path: reference.path().clone(),
            }
            .to_diagnostic()];
        };
        if entry.path() == reference.path() {
            return Vec::new();
        }
        vec![ResourceError::AssetReferencePathMismatch {
            guid: reference.guid(),
            stored_path: reference.path().clone(),
            manifest_path: entry.path().clone(),
        }
        .to_diagnostic()]
    }

    /// Validates raw durable asset reference fields before constructing an asset reference.
    ///
    /// This keeps malformed persisted values reportable without loading source
    /// files, running importers, or assigning replacement identity.
    #[must_use]
    pub fn raw_asset_reference_diagnostics(
        &self,
        raw_guid: u64,
        raw_path: impl Into<String>,
    ) -> Vec<Diagnostic> {
        let path = raw_path.into();
        let guid = match AssetGuid::try_new(raw_guid) {
            Ok(guid) => guid,
            Err(error) => return vec![error.to_diagnostic()],
        };
        let path = match AssetPath::new(path) {
            Ok(path) => path,
            Err(error) => return vec![error.to_diagnostic()],
        };
        self.asset_reference_diagnostics(&AssetReference::new(guid, path))
    }

    /// Validates a durable material reference against manifest identity and material path kind.
    ///
    /// This does not parse `.knmat` source data. It keeps the resource database
    /// responsible for stable identity and path contracts while higher-level
    /// importers own material content validation.
    #[must_use]
    pub fn material_reference_diagnostics(&self, reference: &AssetReference) -> Vec<Diagnostic> {
        let mut diagnostics = self.asset_reference_diagnostics(reference);
        if let Err(error) = validate_asset_kind(reference.path().clone(), AssetImportKind::Material)
        {
            diagnostics.push(error.to_diagnostic());
        }
        diagnostics
    }

    /// Validates raw durable material reference fields before constructing an asset reference.
    ///
    /// Malformed GUIDs or paths are reported directly. Valid raw fields are then
    /// checked against committed manifest identity and material extension rules.
    #[must_use]
    pub fn raw_material_reference_diagnostics(
        &self,
        raw_guid: u64,
        raw_path: impl Into<String>,
    ) -> Vec<Diagnostic> {
        let path = raw_path.into();
        let guid = match AssetGuid::try_new(raw_guid) {
            Ok(guid) => guid,
            Err(error) => return vec![error.to_diagnostic()],
        };
        let path = match AssetPath::new(path) {
            Ok(path) => path,
            Err(error) => return vec![error.to_diagnostic()],
        };
        self.material_reference_diagnostics(&AssetReference::new(guid, path))
    }

    /// Validates reflected scene asset references through this resource database.
    ///
    /// Diagnostics are emitted in deterministic scene traversal and property
    /// path order. Resource diagnostics are enriched with scene instance GUID,
    /// scene path, and reflected property path context.
    #[must_use]
    pub fn scene_asset_reference_diagnostics(&self, scene: &Scene) -> Vec<Diagnostic> {
        let Some(root_id) = scene.root_id() else {
            return Vec::new();
        };
        let mut diagnostics = Vec::new();
        self.collect_scene_asset_reference_diagnostics(scene, root_id, &mut diagnostics);
        diagnostics
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

    fn collect_scene_asset_reference_diagnostics(
        &self,
        scene: &Scene,
        id: kinetik_core::InstanceId,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Ok(instance) = scene.get(id) else {
            return;
        };
        let scene_path = scene.path(id).ok();

        for (property_path, value) in &instance.properties {
            let PropertyValue::AssetReference(reference) = value else {
                continue;
            };
            diagnostics.extend(
                self.asset_reference_value_diagnostics(reference)
                    .into_iter()
                    .map(|mut diagnostic| {
                        diagnostic.location.instance_guid = Some(instance.guid);
                        diagnostic.location.scene_path.clone_from(&scene_path);
                        diagnostic.location.property_path = Some(property_path.clone());
                        diagnostic
                    }),
            );
        }

        for child_id in &instance.children {
            self.collect_scene_asset_reference_diagnostics(scene, *child_id, diagnostics);
        }
    }

    fn asset_reference_value_diagnostics(
        &self,
        reference: &AssetReferenceValue,
    ) -> Vec<Diagnostic> {
        self.raw_asset_reference_diagnostics(reference.guid(), reference.path())
    }
}
