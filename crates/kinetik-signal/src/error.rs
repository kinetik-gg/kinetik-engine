use core::fmt;

use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
    SignalId,
};

use crate::{SignalConnectionId, SignalFlushDomain, SignalOwner};

/// Result type for signal model operations.
pub type SignalResult<T> = Result<T, SignalError>;

/// Errors returned by signal descriptor and connection validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalError {
    /// Author-facing signal name was empty.
    EmptySignalName,
    /// Author-facing signal name was not `PascalCase`.
    InvalidSignalName {
        /// Invalid signal name.
        name: String,
    },
    /// A signal with the same owner and author-facing name already exists.
    DuplicateSignal {
        /// Signal owner where the duplicate was found.
        owner: SignalOwner,
        /// Duplicate author-facing signal name.
        name: String,
    },
    /// Signal ID was not registered.
    UnknownSignal {
        /// Missing signal ID.
        id: SignalId,
    },
    /// Connection ID was not registered.
    UnknownConnection {
        /// Missing connection ID.
        id: SignalConnectionId,
    },
    /// Event was queued through the wrong flush-domain API for the signal.
    WrongFlushDomain {
        /// Target signal ID.
        id: SignalId,
        /// Signal descriptor flush domain.
        expected: SignalFlushDomain,
        /// Requested event flush domain.
        actual: SignalFlushDomain,
    },
}

impl SignalError {
    /// Stable diagnostic code for invalid author-facing signal names.
    pub const INVALID_SIGNAL_NAME_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_SIGNAL_INVALID_NAME");

    /// Stable diagnostic code for duplicate signal names.
    pub const DUPLICATE_SIGNAL_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_SIGNAL_DUPLICATE_SIGNAL");

    /// Stable diagnostic code for invalid signal handles.
    pub const UNKNOWN_SIGNAL_CODE: DiagnosticCode = DiagnosticCode::new("KT_SIGNAL_UNKNOWN_SIGNAL");

    /// Stable diagnostic code for invalid or stale connection handles.
    pub const UNKNOWN_CONNECTION_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_SIGNAL_UNKNOWN_CONNECTION");

    /// Stable diagnostic code for events queued through the wrong flush domain.
    pub const WRONG_FLUSH_DOMAIN_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_SIGNAL_WRONG_FLUSH_DOMAIN");

    /// Diagnostic source for signal-owned validation.
    pub const SIGNAL_SOURCE: DiagnosticSource = DiagnosticSource::new("Signal");

    /// Returns the stable diagnostic code for this signal error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::EmptySignalName | Self::InvalidSignalName { .. } => {
                Self::INVALID_SIGNAL_NAME_CODE
            }
            Self::DuplicateSignal { .. } => Self::DUPLICATE_SIGNAL_CODE,
            Self::UnknownSignal { .. } => Self::UNKNOWN_SIGNAL_CODE,
            Self::UnknownConnection { .. } => Self::UNKNOWN_CONNECTION_CODE,
            Self::WrongFlushDomain { .. } => Self::WRONG_FLUSH_DOMAIN_CODE,
        }
    }

    /// Converts this error into a structured diagnostic.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            Self::SIGNAL_SOURCE,
            self.to_string(),
        )
        .with_blocking_scope(DiagnosticBlockingScope::Play)
    }
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySignalName => f.write_str("signal name must not be empty"),
            Self::InvalidSignalName { name } => {
                write!(f, "signal name must be PascalCase: {name}")
            }
            Self::DuplicateSignal { owner, name } => {
                write!(f, "signal {name} is already registered for {owner}")
            }
            Self::UnknownSignal { id } => write!(f, "signal is not registered: {id}"),
            Self::UnknownConnection { id } => {
                write!(f, "signal connection is not registered: {id}")
            }
            Self::WrongFlushDomain {
                id,
                expected,
                actual,
            } => write!(
                f,
                "signal {id} uses {expected:?} flush domain, not {actual:?}"
            ),
        }
    }
}

impl std::error::Error for SignalError {}
