//! Import foundation integration tests.

use kinetik_core::DiagnosticBlockingScope;
use kinetik_resource::{
    AssetGuid, AssetImportKind, AssetImportRequest, AssetPath, ImportArtifactRecord,
    ImportSettingsHash, ResourceError, SourceContentHash, GLTF_IMPORTER_ID, IMPORTER_VERSION,
    IMPORT_CACHE_SCHEMA_VERSION, MATERIAL_IMPORTER_ID, TEXTURE_IMPORTER_ID,
};

fn request(path: &str, kind: AssetImportKind) -> AssetImportRequest {
    AssetImportRequest::new(
        AssetGuid::new(7),
        AssetPath::new(path).unwrap(),
        kind,
        SourceContentHash::new("source-hash").unwrap(),
        ImportSettingsHash::new("settings-hash").unwrap(),
    )
    .unwrap()
}

#[test]
fn texture_import_request_creates_stable_cache_and_artifact_metadata() {
    let request = request(
        "res://assets/textures/checker.png",
        AssetImportKind::Texture,
    );
    let record = request.to_cache_record().unwrap();
    let artifact = ImportArtifactRecord::from_request(&request).unwrap();

    assert_eq!(request.kind().importer_id(), TEXTURE_IMPORTER_ID);
    assert_eq!(record.asset_guid(), AssetGuid::new(7));
    assert_eq!(record.importer_id(), TEXTURE_IMPORTER_ID);
    assert_eq!(record.importer_version(), IMPORTER_VERSION);
    assert_eq!(record.cache_schema_version(), IMPORT_CACHE_SCHEMA_VERSION);
    assert_eq!(
        artifact.artifact_path(),
        ".kinetik/import/textures/7-source-hash.kasset"
    );
    assert_eq!(artifact.cache_record(), &record);
}

#[test]
fn gltf_import_request_creates_stable_cache_and_artifact_metadata() {
    let request = request(
        "res://assets/models/showcase.glb",
        AssetImportKind::GltfMesh,
    );
    let artifact = ImportArtifactRecord::from_request(&request).unwrap();

    assert_eq!(request.kind().importer_id(), GLTF_IMPORTER_ID);
    assert_eq!(artifact.kind(), AssetImportKind::GltfMesh);
    assert_eq!(
        artifact.artifact_path(),
        ".kinetik/import/meshes/7-source-hash.kasset"
    );
}

#[test]
fn material_import_request_creates_stable_cache_and_artifact_metadata() {
    let request = request(
        "res://assets/materials/base.knmat",
        AssetImportKind::Material,
    );
    let artifact = ImportArtifactRecord::from_request(&request).unwrap();

    assert_eq!(request.kind().importer_id(), MATERIAL_IMPORTER_ID);
    assert_eq!(
        artifact.source_path().as_str(),
        "res://assets/materials/base.knmat"
    );
    assert_eq!(
        artifact.artifact_path(),
        ".kinetik/import/materials/7-source-hash.kasset"
    );
}

#[test]
fn import_request_rejects_path_that_does_not_match_kind() {
    let error = AssetImportRequest::new(
        AssetGuid::new(7),
        AssetPath::new("res://assets/textures/checker.png").unwrap(),
        AssetImportKind::Material,
        SourceContentHash::new("source-hash").unwrap(),
        ImportSettingsHash::new("settings-hash").unwrap(),
    )
    .unwrap_err();

    assert_eq!(
        error,
        ResourceError::InvalidAssetKind {
            path: AssetPath::new("res://assets/textures/checker.png").unwrap(),
            expected: ".knmat",
        }
    );
    let diagnostic = error.to_diagnostic();
    assert_eq!(diagnostic.code, ResourceError::INVALID_ASSET_KIND_CODE);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Import));
    assert_eq!(
        diagnostic.location.asset_path.as_deref(),
        Some("res://assets/textures/checker.png")
    );
}
