use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};

/// Render diagnostic source.
pub const RENDER_SOURCE: DiagnosticSource = DiagnosticSource::new("Render");

/// Diagnostic code for scenes without a camera.
pub const MISSING_CAMERA_CODE: DiagnosticCode = DiagnosticCode::new("KT_RENDER_MISSING_CAMERA");

/// Diagnostic code for scenes without lights.
pub const MISSING_LIGHT_CODE: DiagnosticCode = DiagnosticCode::new("KT_RENDER_MISSING_LIGHT");

/// Diagnostic code for missing or unsupported primitive mesh data.
pub const MISSING_MESH_CODE: DiagnosticCode = DiagnosticCode::new("KT_RENDER_MISSING_MESH");

/// Diagnostic code for missing material data that used a fallback.
pub const MISSING_MATERIAL_CODE: DiagnosticCode = DiagnosticCode::new("KT_RENDER_MISSING_MATERIAL");

/// Builds a structured render diagnostic.
#[must_use]
pub fn render_diagnostic(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        code,
        DiagnosticSeverity::Warning,
        RENDER_SOURCE,
        message.into(),
    )
    .with_blocking_scope(DiagnosticBlockingScope::Play)
}
