//! Resource database integration tests.

use kinetik_core::DiagnosticBlockingScope;
use kinetik_resource::{
    AssetGuid, AssetManifest, AssetManifestEntry, AssetPath, AssetReference, ResourceDatabase,
    ResourceError,
};

fn manifest_entry(raw_guid: u64, path: &str) -> AssetManifestEntry {
    AssetManifestEntry::from_parts(
        AssetGuid::new(raw_guid),
        path,
        "kinetik.test",
        "1",
        format!("hash-{raw_guid}"),
    )
    .unwrap()
}

#[test]
fn resource_database_iterates_manifest_entries_in_deterministic_path_order() {
    let manifest = AssetManifest::from_entries(vec![
        manifest_entry(3, "res://assets/zeta.mesh"),
        manifest_entry(1, "res://assets/alpha.mesh"),
        manifest_entry(2, "res://assets/mid.mesh"),
    ])
    .unwrap();

    let database = ResourceDatabase::from_manifest(manifest);

    assert_eq!(
        database
            .entries()
            .iter()
            .map(|entry| entry.path().as_str())
            .collect::<Vec<_>>(),
        vec![
            "res://assets/alpha.mesh",
            "res://assets/mid.mesh",
            "res://assets/zeta.mesh",
        ]
    );
}

#[test]
fn resource_database_looks_up_entries_by_guid_and_path() {
    let first = manifest_entry(1, "res://assets/alpha.mesh");
    let second = manifest_entry(2, "res://assets/mid.mesh");
    let manifest = AssetManifest::from_entries(vec![first.clone(), second.clone()]).unwrap();
    let database = ResourceDatabase::from_manifest(manifest);
    let second_path = AssetPath::new("res://assets/mid.mesh").unwrap();

    assert_eq!(database.get_by_guid(AssetGuid::new(1)), Some(&first));
    assert_eq!(database.get_by_path(&second_path), Some(&second));
    assert_eq!(
        database.get_by_res_path("res://assets/mid.mesh").unwrap(),
        Some(&second)
    );
    assert!(database
        .get_by_res_path("res://assets/missing.mesh")
        .unwrap()
        .is_none());
}

#[test]
fn resource_database_validates_raw_res_path_lookup() {
    let database = ResourceDatabase::new();

    let error = database.get_by_res_path("assets/missing.mesh").unwrap_err();

    assert_eq!(
        error.diagnostic_code(),
        ResourceError::INVALID_ASSET_PATH_CODE
    );
}

#[test]
fn resource_database_accepts_manifest_matching_asset_references() {
    let manifest = AssetManifest::from_entries(vec![manifest_entry(
        7,
        "res://assets/materials/stone.knmat",
    )])
    .unwrap();
    let database = ResourceDatabase::from_manifest(manifest);
    let reference = AssetReference::new(
        AssetGuid::new(7),
        AssetPath::new("res://assets/materials/stone.knmat").unwrap(),
    );

    assert!(database.asset_reference_diagnostics(&reference).is_empty());
    assert!(database
        .raw_asset_reference_diagnostics(7, "res://assets/materials/stone.knmat")
        .is_empty());
}

#[test]
fn resource_database_reports_missing_asset_reference_guids() {
    let database = ResourceDatabase::new();
    let reference = AssetReference::new(
        AssetGuid::new(9),
        AssetPath::new("res://assets/materials/missing.knmat").unwrap(),
    );

    let diagnostics = database.asset_reference_diagnostics(&reference);

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code, ResourceError::MISSING_ASSET_REFERENCE_CODE);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Save));
    assert_eq!(
        diagnostic.location.asset_path.as_deref(),
        Some("res://assets/materials/missing.knmat")
    );
    assert!(diagnostic.message.contains("AssetGuid(9)"));
}

#[test]
fn resource_database_reports_asset_reference_path_mismatches() {
    let manifest =
        AssetManifest::from_entries(vec![manifest_entry(5, "res://assets/materials/new.knmat")])
            .unwrap();
    let database = ResourceDatabase::from_manifest(manifest);
    let reference = AssetReference::new(
        AssetGuid::new(5),
        AssetPath::new("res://assets/materials/old.knmat").unwrap(),
    );

    let diagnostics = database.asset_reference_diagnostics(&reference);

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        ResourceError::ASSET_REFERENCE_PATH_MISMATCH_CODE
    );
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Save));
    assert_eq!(
        diagnostic.location.asset_path.as_deref(),
        Some("res://assets/materials/old.knmat")
    );
    assert!(diagnostic
        .message
        .contains("res://assets/materials/new.knmat"));
}

#[test]
fn resource_database_reports_raw_asset_reference_field_errors() {
    let database = ResourceDatabase::new();

    let invalid_guid = database.raw_asset_reference_diagnostics(0, "res://assets/a.mesh");
    assert_eq!(invalid_guid.len(), 1);
    assert_eq!(invalid_guid[0].code, ResourceError::INVALID_ASSET_GUID_CODE);

    let invalid_path = database.raw_asset_reference_diagnostics(1, "assets/a.mesh");
    assert_eq!(invalid_path.len(), 1);
    assert_eq!(invalid_path[0].code, ResourceError::INVALID_ASSET_PATH_CODE);
    assert_eq!(
        invalid_path[0].location.asset_path.as_deref(),
        Some("assets/a.mesh")
    );
}

#[test]
fn resource_database_reports_no_missing_sources_when_all_paths_are_observed() {
    let manifest = AssetManifest::from_entries(vec![
        manifest_entry(1, "res://assets/alpha.mesh"),
        manifest_entry(2, "res://assets/mid.mesh"),
    ])
    .unwrap();
    let database = ResourceDatabase::from_manifest(manifest);
    let observed_paths = vec![
        AssetPath::new("res://assets/mid.mesh").unwrap(),
        AssetPath::new("res://assets/alpha.mesh").unwrap(),
    ];

    assert!(database
        .missing_source_diagnostics(observed_paths)
        .is_empty());
}

#[test]
fn resource_database_reports_missing_sources_in_manifest_order() {
    let manifest = AssetManifest::from_entries(vec![
        manifest_entry(3, "res://assets/zeta.mesh"),
        manifest_entry(1, "res://assets/alpha.mesh"),
        manifest_entry(2, "res://assets/mid.mesh"),
    ])
    .unwrap();
    let database = ResourceDatabase::from_manifest(manifest);
    let observed_paths = vec![AssetPath::new("res://assets/mid.mesh").unwrap()];

    let diagnostics = database.missing_source_diagnostics(observed_paths);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.location.asset_path.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("res://assets/alpha.mesh"),
            Some("res://assets/zeta.mesh"),
        ]
    );
}

#[test]
fn resource_database_missing_source_diagnostic_has_stable_shape() {
    let manifest =
        AssetManifest::from_entries(vec![manifest_entry(7, "res://assets/missing.mesh")]).unwrap();
    let database = ResourceDatabase::from_manifest(manifest);

    let diagnostic = database
        .missing_source_diagnostics(Vec::<AssetPath>::new())
        .pop()
        .unwrap();

    assert_eq!(diagnostic.code, ResourceError::MISSING_SOURCE_ASSET_CODE);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Import));
    assert_eq!(
        diagnostic.location.asset_path.as_deref(),
        Some("res://assets/missing.mesh")
    );
    assert!(diagnostic.message.contains("AssetGuid(7)"));
}
