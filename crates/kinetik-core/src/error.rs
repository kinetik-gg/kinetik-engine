use core::fmt;

use crate::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};

/// Standard result type used by Kinetik crates.
pub type KinetikResult<T> = Result<T, KinetikError>;

/// Foundational engine error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KinetikError {
    /// A handle or ID was not valid in the receiving system.
    InvalidHandle {
        /// Human-readable handle kind, such as `InstanceId`.
        kind: &'static str,
        /// Raw handle value that failed validation.
        id: u64,
    },
    /// A requested item was not found.
    NotFound {
        /// Human-readable item kind, such as `Resource`.
        kind: &'static str,
        /// Requested item name or path.
        name: String,
    },
    /// The operation is not implemented yet.
    NotImplemented {
        /// Feature name that is not implemented yet.
        feature: &'static str,
    },
}

impl fmt::Display for KinetikError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle { kind, id } => write!(f, "invalid {kind} handle: {id}"),
            Self::NotFound { kind, name } => write!(f, "{kind} not found: {name}"),
            Self::NotImplemented { feature } => write!(f, "feature not implemented: {feature}"),
        }
    }
}

impl std::error::Error for KinetikError {}

impl KinetikError {
    /// Returns the stable diagnostic code for this error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::InvalidHandle { .. } => DiagnosticCode::CORE_INVALID_HANDLE,
            Self::NotFound { .. } => DiagnosticCode::CORE_NOT_FOUND,
            Self::NotImplemented { .. } => DiagnosticCode::CORE_NOT_IMPLEMENTED,
        }
    }

    /// Converts this error into a structured diagnostic with the provided source.
    #[must_use]
    pub fn to_diagnostic(
        &self,
        source: DiagnosticSource,
        blocking: Option<DiagnosticBlockingScope>,
    ) -> Diagnostic {
        let mut diagnostic = Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            source,
            self.to_string(),
        );
        diagnostic.blocking = blocking;
        diagnostic
    }
}

impl From<KinetikError> for Diagnostic {
    fn from(error: KinetikError) -> Self {
        error.to_diagnostic(DiagnosticSource::CORE, None)
    }
}
