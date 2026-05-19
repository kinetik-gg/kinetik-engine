//! Project identity, settings, document references, and health contracts.
//!
//! This crate owns source project state that is broader than assets/resources
//! and narrower than editor session state.

use core::fmt;

use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};
use kinetik_resource::{ProjectLayout, ProjectPathKind, ResourceError};

/// Result type for project model operations.
pub type ProjectResult<T> = Result<T, ProjectError>;

/// Errors returned by project identity and settings validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectError {
    /// Project display name was empty.
    EmptyProjectName,
    /// Engine compatibility string was empty.
    EmptyEngineCompatibility,
    /// Workspace-relative document path was empty.
    EmptyDocumentPath {
        /// Logical document path field.
        field: &'static str,
    },
}

impl ProjectError {
    /// Stable diagnostic code for empty project names.
    pub const EMPTY_PROJECT_NAME_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_PROJECT_EMPTY_NAME");

    /// Stable diagnostic code for empty engine compatibility.
    pub const EMPTY_ENGINE_COMPATIBILITY_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_PROJECT_EMPTY_ENGINE_COMPATIBILITY");

    /// Stable diagnostic code for empty document references.
    pub const EMPTY_DOCUMENT_PATH_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_PROJECT_EMPTY_DOCUMENT_PATH");

    /// Diagnostic source for project-owned validation.
    pub const PROJECT_SOURCE: DiagnosticSource = DiagnosticSource::new("Project");

    /// Returns the stable diagnostic code for this project error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::EmptyProjectName => Self::EMPTY_PROJECT_NAME_CODE,
            Self::EmptyEngineCompatibility => Self::EMPTY_ENGINE_COMPATIBILITY_CODE,
            Self::EmptyDocumentPath { .. } => Self::EMPTY_DOCUMENT_PATH_CODE,
        }
    }

    /// Converts this error into a structured diagnostic.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            Self::PROJECT_SOURCE,
            self.to_string(),
        )
        .with_blocking_scope(DiagnosticBlockingScope::Build)
    }
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProjectName => f.write_str("project name must not be empty"),
            Self::EmptyEngineCompatibility => {
                f.write_str("project engine compatibility must not be empty")
            }
            Self::EmptyDocumentPath { field } => {
                write!(f, "project document path must not be empty: {field}")
            }
        }
    }
}

impl std::error::Error for ProjectError {}

/// Project identity fields owned by the future `Kinetik.toml` contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectIdentity {
    name: String,
    engine_compatibility: String,
}

impl ProjectIdentity {
    /// Creates a project identity after validating required text fields.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError`] when required identity fields are empty.
    pub fn new(
        name: impl Into<String>,
        engine_compatibility: impl Into<String>,
    ) -> ProjectResult<Self> {
        let name = validate_required_text("name", name.into())?;
        let engine_compatibility =
            validate_required_text("engine_compatibility", engine_compatibility.into())?;
        Ok(Self {
            name,
            engine_compatibility,
        })
    }

    /// Returns the project display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the engine compatibility string.
    #[must_use]
    pub fn engine_compatibility(&self) -> &str {
        &self.engine_compatibility
    }
}

/// Dependency-free future `Kinetik.toml` settings document contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSettingsDocument {
    identity: ProjectIdentity,
}

impl ProjectSettingsDocument {
    /// Creates project settings from validated identity.
    #[must_use]
    pub const fn new(identity: ProjectIdentity) -> Self {
        Self { identity }
    }

    /// Returns project identity settings.
    #[must_use]
    pub const fn identity(&self) -> &ProjectIdentity {
        &self.identity
    }
}

/// Workspace-relative source document references owned by project state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDocumentRefs {
    active_scene: String,
    assets_manifest: String,
    instances_manifest: String,
}

impl ProjectDocumentRefs {
    /// Creates document references after validating required paths.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError`] when any document path is empty.
    pub fn new(
        active_scene: impl Into<String>,
        assets_manifest: impl Into<String>,
        instances_manifest: impl Into<String>,
    ) -> ProjectResult<Self> {
        Ok(Self {
            active_scene: validate_document_path("active_scene", active_scene.into())?,
            assets_manifest: validate_document_path("assets_manifest", assets_manifest.into())?,
            instances_manifest: validate_document_path(
                "instances_manifest",
                instances_manifest.into(),
            )?,
        })
    }

    /// Creates the default project document references from the canonical layout.
    ///
    /// # Panics
    ///
    /// Panics only if the built-in project layout no longer contains required
    /// project document paths.
    #[must_use]
    pub fn from_default_layout() -> Self {
        let layout = ProjectLayout::new();
        Self::new(
            layout
                .path(ProjectPathKind::MainScene)
                .expect("default layout should contain a main scene path"),
            layout
                .path(ProjectPathKind::AssetsManifest)
                .expect("default layout should contain an assets manifest path"),
            layout
                .path(ProjectPathKind::InstancesManifest)
                .expect("default layout should contain an instances manifest path"),
        )
        .expect("default layout document paths should be valid")
    }

    /// Returns the active scene source path.
    #[must_use]
    pub fn active_scene(&self) -> &str {
        &self.active_scene
    }

    /// Returns the asset manifest source path.
    #[must_use]
    pub fn assets_manifest(&self) -> &str {
        &self.assets_manifest
    }

    /// Returns the instance manifest source path.
    #[must_use]
    pub fn instances_manifest(&self) -> &str {
        &self.instances_manifest
    }
}

impl Default for ProjectDocumentRefs {
    fn default() -> Self {
        Self::from_default_layout()
    }
}

/// Deterministic current-health diagnostics store for project state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticsStore {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsStore {
    /// Creates an empty diagnostics store.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Creates a diagnostics store from deterministic diagnostic order.
    #[must_use]
    pub const fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    /// Adds a diagnostic to the store.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Clears all diagnostics.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// Returns all diagnostics in deterministic insertion order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns the number of current diagnostics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns whether the store has no diagnostics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns diagnostics matching `severity` in deterministic order.
    #[must_use]
    pub fn by_severity(&self, severity: DiagnosticSeverity) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == severity)
            .collect()
    }

    /// Returns diagnostics matching `blocking` in deterministic order.
    #[must_use]
    pub fn by_blocking_scope(&self, blocking: DiagnosticBlockingScope) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.blocking == Some(blocking))
            .collect()
    }
}

/// Engine-owned project source model without editor-only session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModel {
    settings: ProjectSettingsDocument,
    documents: ProjectDocumentRefs,
    layout: ProjectLayout,
    diagnostics: DiagnosticsStore,
}

impl ProjectModel {
    /// Creates a project model without running workspace layout validation.
    #[must_use]
    pub fn new(settings: ProjectSettingsDocument, documents: ProjectDocumentRefs) -> Self {
        Self {
            settings,
            documents,
            layout: ProjectLayout::new(),
            diagnostics: DiagnosticsStore::new(),
        }
    }

    /// Creates a project model and records layout diagnostics from present paths.
    pub fn with_layout_validation<I, P>(
        settings: ProjectSettingsDocument,
        documents: ProjectDocumentRefs,
        present_paths: I,
    ) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        let mut model = Self::new(settings, documents);
        model.validate_layout(present_paths);
        model
    }

    /// Validates project layout paths and replaces current project diagnostics.
    pub fn validate_layout<I, P>(&mut self, present_paths: I)
    where
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        self.diagnostics.clear();
        if let Err(error) = self.layout.validate_required_paths(present_paths) {
            self.diagnostics.push(resource_error_to_diagnostic(&error));
        }
    }

    /// Returns project settings.
    #[must_use]
    pub const fn settings(&self) -> &ProjectSettingsDocument {
        &self.settings
    }

    /// Returns active project document references.
    #[must_use]
    pub const fn documents(&self) -> &ProjectDocumentRefs {
        &self.documents
    }

    /// Returns the canonical project layout model.
    #[must_use]
    pub const fn layout(&self) -> &ProjectLayout {
        &self.layout
    }

    /// Returns project health diagnostics.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticsStore {
        &self.diagnostics
    }
}

fn resource_error_to_diagnostic(error: &ResourceError) -> Diagnostic {
    error.to_diagnostic()
}

fn validate_required_text(field: &'static str, value: String) -> ProjectResult<String> {
    if !value.trim().is_empty() {
        return Ok(value);
    }
    match field {
        "name" => Err(ProjectError::EmptyProjectName),
        "engine_compatibility" => Err(ProjectError::EmptyEngineCompatibility),
        _ => Err(ProjectError::EmptyDocumentPath { field }),
    }
}

fn validate_document_path(field: &'static str, value: String) -> ProjectResult<String> {
    if value.trim().is_empty() {
        return Err(ProjectError::EmptyDocumentPath { field });
    }
    Ok(value)
}

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-project"
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let warning = Diagnostic::new(
            DiagnosticCode::new("KT_TEST_WARNING"),
            DiagnosticSeverity::Warning,
            DiagnosticSource::new("Test"),
            "warning",
        )
        .with_blocking_scope(DiagnosticBlockingScope::Edit);
        let error = Diagnostic::new(
            DiagnosticCode::new("KT_TEST_ERROR"),
            DiagnosticSeverity::Error,
            DiagnosticSource::new("Test"),
            "error",
        )
        .with_blocking_scope(DiagnosticBlockingScope::Build);
        let store = DiagnosticsStore::from_diagnostics(vec![warning.clone(), error.clone()]);

        assert_eq!(store.by_severity(DiagnosticSeverity::Error), vec![&error]);
        assert_eq!(
            store.by_blocking_scope(DiagnosticBlockingScope::Edit),
            vec![&warning]
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
        assert_eq!(model.diagnostics().len(), 1);

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

        assert!(model.diagnostics().is_empty());
    }
}
