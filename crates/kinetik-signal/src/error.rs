use core::fmt;

use kinetik_core::SignalId;

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
