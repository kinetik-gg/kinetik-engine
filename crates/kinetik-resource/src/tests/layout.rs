use super::*;

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
