use super::*;

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
        AssetManifestEntry::from_parts(AssetGuid::new(1), "", "gltf", "1.0.0", "hash").unwrap_err(),
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
        manifest_entry(3, "res://assets/materials/bark.knmat", "material").unwrap(),
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
            "res://assets/materials/bark.knmat",
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
