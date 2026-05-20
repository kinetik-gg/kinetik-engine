use super::*;

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
