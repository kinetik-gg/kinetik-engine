//! Resource handles and asset path contracts for Kinetik.

mod cache;
mod database;
mod error;
mod identity;
mod import;
mod layout;
mod manifest;

pub use cache::{ImportCacheRecord, ImportCacheSchemaVersion, SourceContentHash};
pub use database::ResourceDatabase;
pub use error::{ResourceError, ResourceResult};
pub use identity::{AssetGuid, AssetPath, AssetReference, ResourceHandle};
pub use import::{
    validate_asset_kind, AssetImportKind, AssetImportRequest, ImportArtifactRecord,
    GLTF_IMPORTER_ID, IMPORTER_VERSION, IMPORT_CACHE_SCHEMA_VERSION, MATERIAL_IMPORTER_ID,
    TEXTURE_IMPORTER_ID,
};
pub use layout::{ProjectLayout, ProjectLayoutPath, ProjectPathDomain, ProjectPathKind};
pub use manifest::{
    AssetManifest, AssetManifestDocument, AssetManifestEntry, AssetManifestEntryDocument,
    ImportSettingsHash,
};

#[cfg(test)]
mod tests;
