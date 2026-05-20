//! Import cache metadata integration tests.

use kinetik_resource::{
    AssetGuid, ImportCacheRecord, ImportCacheSchemaVersion, ImportSettingsHash, ResourceError,
    SourceContentHash,
};

#[test]
fn import_cache_record_stores_stable_metadata() {
    let record = ImportCacheRecord::new(
        AssetGuid::new(7),
        SourceContentHash::new("source-hash").unwrap(),
        "kinetik.gltf",
        "1.0.0",
        ImportSettingsHash::new("settings-hash").unwrap(),
        ImportCacheSchemaVersion::new(1),
    )
    .unwrap();

    assert_eq!(record.asset_guid(), AssetGuid::new(7));
    assert_eq!(record.source_content_hash().as_str(), "source-hash");
    assert_eq!(record.importer_id(), "kinetik.gltf");
    assert_eq!(record.importer_version(), "1.0.0");
    assert_eq!(record.settings_hash().as_str(), "settings-hash");
    assert_eq!(record.cache_schema_version().raw(), 1);
}

#[test]
fn import_cache_metadata_rejects_empty_required_text() {
    assert_eq!(
        SourceContentHash::new(" ").unwrap_err(),
        ResourceError::InvalidImporterMetadata {
            field: "source_content_hash",
            value: " ".to_owned(),
        }
    );

    let error = ImportCacheRecord::new(
        AssetGuid::new(7),
        SourceContentHash::new("source-hash").unwrap(),
        "kinetik.gltf",
        " ",
        ImportSettingsHash::new("settings-hash").unwrap(),
        ImportCacheSchemaVersion::new(1),
    )
    .unwrap_err();

    assert_eq!(
        error,
        ResourceError::InvalidImporterMetadata {
            field: "version",
            value: " ".to_owned(),
        }
    );
}

#[test]
fn import_cache_schema_version_rejects_zero() {
    assert_eq!(
        ImportCacheSchemaVersion::try_new(0).unwrap_err(),
        ResourceError::InvalidImporterMetadata {
            field: "cache_schema_version",
            value: "0".to_owned(),
        }
    );
    assert!(std::panic::catch_unwind(|| ImportCacheSchemaVersion::new(0)).is_err());
}
