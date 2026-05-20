//! Resource database integration tests.

use kinetik_resource::{
    AssetGuid, AssetManifest, AssetManifestEntry, AssetPath, ResourceDatabase, ResourceError,
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
