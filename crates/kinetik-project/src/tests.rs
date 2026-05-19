use super::*;
use kinetik_core::DiagnosticLocation;
use kinetik_resource::ResourceError;

fn settings() -> ProjectSettingsDocument {
    ProjectSettingsDocument::new(ProjectIdentity::new("Example", "0.0").unwrap())
}

#[test]
fn exposes_crate_name() {
    assert_eq!(crate_name(), "kinetik-project");
}

#[test]
fn project_identity_rejects_empty_required_fields() {
    assert_eq!(
        ProjectIdentity::new(" ", "0.0").unwrap_err(),
        ProjectError::EmptyProjectName
    );
    assert_eq!(
        ProjectIdentity::new("Example", " ").unwrap_err(),
        ProjectError::EmptyEngineCompatibility
    );
}

#[test]
fn project_errors_convert_to_build_diagnostics() {
    let diagnostic = ProjectError::EmptyProjectName.to_diagnostic();

    assert_eq!(diagnostic.code, ProjectError::EMPTY_PROJECT_NAME_CODE);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.source, ProjectError::PROJECT_SOURCE);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Build));
}

#[test]
fn default_document_refs_follow_canonical_project_layout() {
    let refs = ProjectDocumentRefs::from_default_layout();

    assert_eq!(refs.active_scene(), "scenes/main.ktscene");
    assert_eq!(refs.assets_manifest(), "project/assets.ktmanifest");
    assert_eq!(refs.instances_manifest(), "project/instances.ktmanifest");
}

#[test]
fn document_refs_reject_empty_paths() {
    assert_eq!(
        ProjectDocumentRefs::new(
            "",
            "project/assets.ktmanifest",
            "project/instances.ktmanifest"
        )
        .unwrap_err(),
        ProjectError::EmptyDocumentPath {
            field: "active_scene"
        }
    );
}

#[test]
fn diagnostics_store_filters_without_reordering() {
    let source = DiagnosticSource::new("Test");
    let warning = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_WARNING"),
        DiagnosticSeverity::Warning,
        source,
        "warning",
    )
    .with_blocking_scope(DiagnosticBlockingScope::Edit);
    let error = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_ERROR"),
        DiagnosticSeverity::Error,
        source,
        "error",
    )
    .with_blocking_scope(DiagnosticBlockingScope::Build);
    let store = DiagnosticsStore::from_diagnostics(vec![warning.clone(), error.clone()]);

    assert_eq!(store.by_severity(DiagnosticSeverity::Error), vec![&error]);
    assert_eq!(store.by_source(source), vec![&warning, &error]);
    assert_eq!(
        store.by_blocking_scope(DiagnosticBlockingScope::Edit),
        vec![&warning]
    );
}

#[test]
fn diagnostics_store_replaces_source_owned_current_health() {
    let layout = ResourceError::RESOURCE_SOURCE;
    let settings = ProjectError::PROJECT_SOURCE;
    let stale_layout = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_STALE_LAYOUT"),
        DiagnosticSeverity::Error,
        layout,
        "stale layout",
    );
    let settings_diagnostic = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_SETTINGS"),
        DiagnosticSeverity::Error,
        settings,
        "settings",
    );
    let fresh_layout = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_FRESH_LAYOUT"),
        DiagnosticSeverity::Warning,
        layout,
        "fresh layout",
    );
    let mut store =
        DiagnosticsStore::from_diagnostics(vec![stale_layout, settings_diagnostic.clone()]);

    store.replace_source(layout, [fresh_layout.clone()]);

    assert_eq!(store.diagnostics(), &[settings_diagnostic, fresh_layout]);
}

#[test]
fn diagnostics_store_clears_source_without_touching_other_sources() {
    let layout = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_LAYOUT"),
        DiagnosticSeverity::Error,
        ResourceError::RESOURCE_SOURCE,
        "layout",
    );
    let settings = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_SETTINGS"),
        DiagnosticSeverity::Error,
        ProjectError::PROJECT_SOURCE,
        "settings",
    );
    let mut store = DiagnosticsStore::from_diagnostics(vec![layout, settings.clone()]);

    store.clear_source(ResourceError::RESOURCE_SOURCE);

    assert_eq!(store.diagnostics(), &[settings]);
}

#[test]
fn diagnostics_store_queries_locations_and_repairability() {
    let mut location = DiagnosticLocation::new();
    location.instance_guid = Some(InstanceGuid::new(7));
    location.scene_path = Some("/Game/Enemy".to_owned());
    location.asset_path = Some("res://assets/enemy.glb".to_owned());
    location.script_path = Some("scripts/enemy.luau".to_owned());
    location.property_path = Some("Transform.Position".to_owned());
    let repairable = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_TARGETED"),
        DiagnosticSeverity::Error,
        DiagnosticSource::new("Test"),
        "targeted",
    )
    .with_location(location)
    .allow_agent_repair();
    let other = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_OTHER"),
        DiagnosticSeverity::Info,
        DiagnosticSource::new("Other"),
        "other",
    );
    let store = DiagnosticsStore::from_diagnostics(vec![repairable.clone(), other]);

    assert_eq!(
        store.by_instance_guid(InstanceGuid::new(7)),
        vec![&repairable]
    );
    assert_eq!(
        store.by_asset_path("res://assets/enemy.glb"),
        vec![&repairable]
    );
    assert_eq!(store.by_scene_path("/Game/Enemy"), vec![&repairable]);
    assert_eq!(
        store.by_script_path("scripts/enemy.luau"),
        vec![&repairable]
    );
    assert_eq!(
        store.by_property_path("Transform.Position"),
        vec![&repairable]
    );
    assert_eq!(
        store.by_agent_repair(AgentRepair::Allowed),
        vec![&repairable]
    );
}

#[test]
fn project_model_stores_identity_and_document_refs_without_editor_state() {
    let model = ProjectModel::new(settings(), ProjectDocumentRefs::default());

    assert_eq!(model.settings().identity().name(), "Example");
    assert_eq!(model.documents().active_scene(), "scenes/main.ktscene");
    assert!(model.diagnostics().is_empty());
}

#[test]
fn project_model_records_layout_validation_diagnostics() {
    let model = ProjectModel::with_layout_validation(
        settings(),
        ProjectDocumentRefs::default(),
        ["Kinetik.toml", "assets"],
    );

    let diagnostics = model.diagnostics().diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ResourceError::MISSING_PROJECT_PATHS_CODE
    );
    assert_eq!(
        diagnostics[0].blocking,
        Some(DiagnosticBlockingScope::Build)
    );
}

#[test]
fn project_model_clears_stale_layout_diagnostics_after_valid_layout() {
    let mut model = ProjectModel::with_layout_validation(
        settings(),
        ProjectDocumentRefs::default(),
        ["Kinetik.toml"],
    );
    model
        .diagnostics
        .push(ProjectError::EmptyProjectName.to_diagnostic());
    assert_eq!(model.diagnostics().len(), 2);

    model.validate_layout([
        "Kinetik.toml",
        "scenes",
        "scenes/main.ktscene",
        "prefabs",
        "scripts",
        "assets",
        "project",
        "project/assets.ktmanifest",
        "project/instances.ktmanifest",
    ]);

    assert_eq!(
        model
            .diagnostics()
            .by_source(ProjectError::PROJECT_SOURCE)
            .len(),
        1
    );
    assert!(model
        .diagnostics()
        .by_source(ResourceError::RESOURCE_SOURCE)
        .is_empty());
}
