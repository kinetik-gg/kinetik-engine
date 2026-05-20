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
            "scenes/main.knscene",
            "prefabs",
            "scripts",
            "assets",
            "project",
            "project/assets.knmanifest",
            "project/instances.knmanifest",
            ".kinetik",
            ".kinetik/cache",
            ".kinetik/import",
            ".kinetik/build"
        ]
    );
    assert_eq!(
        layout.path(ProjectPathKind::AssetsManifest),
        Some("project/assets.knmanifest")
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
            "scenes/main.knscene",
            "prefabs",
            "scripts",
            "assets",
            "project",
            "project/assets.knmanifest",
            "project/instances.knmanifest"
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
        "scenes/main.knscene",
        "prefabs",
        "scripts",
        "assets",
        "project",
        "project/assets.knmanifest",
        "project/instances.knmanifest",
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
                "scenes/main.knscene".to_owned(),
                "prefabs".to_owned(),
                "scripts".to_owned(),
                "project".to_owned(),
                "project/assets.knmanifest".to_owned(),
                "project/instances.knmanifest".to_owned()
            ]
        }
    );

    let diagnostic = error.to_diagnostic();
    assert_eq!(diagnostic.code, ResourceError::MISSING_PROJECT_PATHS_CODE);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Build));
}
