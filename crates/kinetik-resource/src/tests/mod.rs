use super::*;

use kinetik_core::{DiagnosticBlockingScope, DiagnosticSeverity};

fn manifest_entry(guid: u64, path: &str, importer_id: &str) -> ResourceResult<AssetManifestEntry> {
    AssetManifestEntry::from_parts(
        AssetGuid::new(guid),
        path,
        importer_id,
        "1.0.0",
        "settings-hash",
    )
}

mod identity;
mod layout;
mod manifest;
