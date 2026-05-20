//! Resource handles and asset path contracts for Kinetik.

mod database;
mod error;
mod identity;
mod layout;
mod manifest;

pub use database::ResourceDatabase;
pub use error::{ResourceError, ResourceResult};
pub use identity::{AssetGuid, AssetPath, AssetReference, ResourceHandle};
pub use layout::{ProjectLayout, ProjectLayoutPath, ProjectPathDomain, ProjectPathKind};
pub use manifest::{
    AssetManifest, AssetManifestDocument, AssetManifestEntry, AssetManifestEntryDocument,
    ImportSettingsHash,
};

#[cfg(test)]
mod tests;
