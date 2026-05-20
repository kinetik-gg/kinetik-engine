//! Shared test fixtures and assertions for Kinetik crates.

use kinetik_core::{
    BundleId, Diagnostic, DiagnosticCode, InstanceGuid, InstanceId, KinetikError, ResourceId,
    ScriptId, SignalId,
};
use kinetik_project::{
    ProjectDocumentRefs, ProjectIdentity, ProjectModel, ProjectSettingsDocument,
};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-test"
}

/// Deterministic project name used by project fixtures.
pub const TEST_PROJECT_NAME: &str = "Test Project";

/// Deterministic engine compatibility string used by project fixtures.
pub const TEST_ENGINE_COMPATIBILITY: &str = "0.0";

/// Deterministic typed ID factory for tests.
///
/// The factory starts at raw value `1` by default because Kinetik reserves zero
/// as invalid for every typed ID kind.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TestIdFactory {
    next_raw: u64,
}

impl TestIdFactory {
    /// Creates a factory that starts at raw value `1`.
    #[must_use]
    pub const fn new() -> Self {
        Self { next_raw: 1 }
    }

    /// Creates a factory that starts at `next_raw`.
    ///
    /// # Panics
    ///
    /// Panics when `next_raw` is zero.
    #[must_use]
    pub const fn starting_at(next_raw: u64) -> Self {
        assert!(next_raw != 0, "test ID factory cannot start at zero");
        Self { next_raw }
    }

    /// Returns the next raw ID value that will be issued.
    #[must_use]
    pub const fn next_raw(self) -> u64 {
        self.next_raw
    }

    /// Returns the next deterministic runtime instance ID.
    #[must_use]
    pub fn instance_id(&mut self) -> InstanceId {
        InstanceId::new(self.take_raw())
    }

    /// Returns the next deterministic serialized instance GUID surrogate.
    #[must_use]
    pub fn instance_guid(&mut self) -> InstanceGuid {
        InstanceGuid::new(self.take_raw())
    }

    /// Returns the next deterministic runtime resource ID.
    #[must_use]
    pub fn resource_id(&mut self) -> ResourceId {
        ResourceId::new(self.take_raw())
    }

    /// Returns the next deterministic runtime signal ID.
    #[must_use]
    pub fn signal_id(&mut self) -> SignalId {
        SignalId::new(self.take_raw())
    }

    /// Returns the next deterministic runtime script ID.
    #[must_use]
    pub fn script_id(&mut self) -> ScriptId {
        ScriptId::new(self.take_raw())
    }

    /// Returns the next deterministic runtime bundle ID.
    #[must_use]
    pub fn bundle_id(&mut self) -> BundleId {
        BundleId::new(self.take_raw())
    }

    fn take_raw(&mut self) -> u64 {
        let raw = self.next_raw;
        self.next_raw = self
            .next_raw
            .checked_add(1)
            .expect("test ID factory exhausted u64 IDs");
        raw
    }
}

impl Default for TestIdFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a deterministic runtime instance ID from a raw value.
///
/// # Panics
///
/// Panics when `raw` is zero.
#[must_use]
pub const fn instance_id(raw: u64) -> InstanceId {
    InstanceId::new(raw)
}

/// Creates a deterministic instance GUID surrogate from a raw value.
///
/// # Panics
///
/// Panics when `raw` is zero.
#[must_use]
pub const fn instance_guid(raw: u64) -> InstanceGuid {
    InstanceGuid::new(raw)
}

/// Creates a deterministic resource ID from a raw value.
///
/// # Panics
///
/// Panics when `raw` is zero.
#[must_use]
pub const fn resource_id(raw: u64) -> ResourceId {
    ResourceId::new(raw)
}

/// Creates a deterministic signal ID from a raw value.
///
/// # Panics
///
/// Panics when `raw` is zero.
#[must_use]
pub const fn signal_id(raw: u64) -> SignalId {
    SignalId::new(raw)
}

/// Creates a deterministic script ID from a raw value.
///
/// # Panics
///
/// Panics when `raw` is zero.
#[must_use]
pub const fn script_id(raw: u64) -> ScriptId {
    ScriptId::new(raw)
}

/// Creates a deterministic bundle ID from a raw value.
///
/// # Panics
///
/// Panics when `raw` is zero.
#[must_use]
pub const fn bundle_id(raw: u64) -> BundleId {
    BundleId::new(raw)
}

/// Creates deterministic project identity fixture data.
///
/// # Panics
///
/// Panics only if the built-in fixture constants become invalid.
#[must_use]
pub fn project_identity() -> ProjectIdentity {
    ProjectIdentity::new(TEST_PROJECT_NAME, TEST_ENGINE_COMPATIBILITY)
        .expect("test project identity should be valid")
}

/// Creates deterministic project settings fixture data.
#[must_use]
pub fn project_settings() -> ProjectSettingsDocument {
    ProjectSettingsDocument::new(project_identity())
}

/// Creates deterministic project document reference fixture data.
#[must_use]
pub fn project_document_refs() -> ProjectDocumentRefs {
    ProjectDocumentRefs::default()
}

/// Creates a deterministic project model fixture without layout diagnostics.
#[must_use]
pub fn project_model() -> ProjectModel {
    ProjectModel::new(project_settings(), project_document_refs())
}

/// Creates a deterministic project model fixture with valid layout diagnostics.
#[must_use]
pub fn project_model_with_valid_layout() -> ProjectModel {
    ProjectModel::with_layout_validation(
        project_settings(),
        project_document_refs(),
        valid_project_paths(),
    )
}

/// Returns canonical project paths that satisfy required source layout validation.
#[must_use]
pub fn valid_project_paths() -> Vec<&'static str> {
    project_model()
        .layout()
        .required_source_paths()
        .into_iter()
        .map(|path| path.path)
        .collect()
}

/// Asserts that a diagnostic has the expected stable code.
///
/// # Panics
///
/// Panics when the diagnostic code does not match `expected`.
pub fn assert_diagnostic_code(diagnostic: &Diagnostic, expected: DiagnosticCode) {
    assert_eq!(
        diagnostic.code, expected,
        "unexpected diagnostic code for message: {}",
        diagnostic.message
    );
}

/// Asserts that an error is an invalid-handle error with the expected kind and raw ID.
///
/// # Panics
///
/// Panics when `error` is not the expected invalid-handle error.
pub fn assert_invalid_handle(error: &KinetikError, expected_kind: &'static str, expected_id: u64) {
    assert_eq!(
        error,
        &KinetikError::InvalidHandle {
            kind: expected_kind,
            id: expected_id,
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetik_core::{DiagnosticSeverity, DiagnosticSource};

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "kinetik-test");
    }

    #[test]
    fn deterministic_id_factory_issues_nonzero_ids_in_order() {
        let mut ids = TestIdFactory::new();

        assert_eq!(ids.instance_id().raw(), 1);
        assert_eq!(ids.instance_guid().raw(), 2);
        assert_eq!(ids.resource_id().raw(), 3);
        assert_eq!(ids.signal_id().raw(), 4);
        assert_eq!(ids.script_id().raw(), 5);
        assert_eq!(ids.bundle_id().raw(), 6);
        assert_eq!(ids.next_raw(), 7);
    }

    #[test]
    fn deterministic_id_factory_can_start_at_custom_nonzero_value() {
        let mut ids = TestIdFactory::starting_at(41);

        assert_eq!(ids.instance_id(), instance_id(41));
        assert_eq!(ids.resource_id(), resource_id(42));
        assert_eq!(ids.next_raw(), 43);
    }

    #[test]
    fn typed_id_helpers_preserve_raw_values() {
        assert_eq!(instance_id(11).raw(), 11);
        assert_eq!(instance_guid(12).raw(), 12);
        assert_eq!(resource_id(13).raw(), 13);
        assert_eq!(signal_id(14).raw(), 14);
        assert_eq!(script_id(15).raw(), 15);
        assert_eq!(bundle_id(16).raw(), 16);
    }

    #[test]
    fn project_fixtures_create_deterministic_project_state() {
        let model = project_model();

        assert_eq!(model.settings().identity().name(), TEST_PROJECT_NAME);
        assert_eq!(
            model.settings().identity().engine_compatibility(),
            TEST_ENGINE_COMPATIBILITY
        );
        assert_eq!(model.documents().active_scene(), "scenes/main.knscene");
        assert!(model.diagnostics().is_empty());
    }

    #[test]
    fn project_layout_fixture_paths_validate_without_diagnostics() {
        let paths = valid_project_paths();
        let model = project_model_with_valid_layout();

        assert!(paths.contains(&"Kinetik.toml"));
        assert!(paths.contains(&"project/assets.knmanifest"));
        assert!(model.diagnostics().is_empty());
    }

    #[test]
    fn assertion_helpers_check_diagnostics_and_invalid_handles() {
        let diagnostic = Diagnostic::new(
            DiagnosticCode::CORE_INVALID_HANDLE,
            DiagnosticSeverity::Error,
            DiagnosticSource::CORE,
            "invalid handle",
        );
        assert_diagnostic_code(&diagnostic, DiagnosticCode::CORE_INVALID_HANDLE);

        let error = KinetikError::InvalidHandle {
            kind: "InstanceId",
            id: 99,
        };
        assert_invalid_handle(&error, "InstanceId", 99);
    }

    #[test]
    fn id_factory_rejects_zero_start() {
        assert!(std::panic::catch_unwind(|| TestIdFactory::starting_at(0)).is_err());
    }
}
