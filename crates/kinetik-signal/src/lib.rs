//! Deterministic signal bus contracts for Kinetik.

use core::{fmt, num::NonZeroU64};

use kinetik_core::{InstanceId, SignalId};

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
        }
    }
}

impl std::error::Error for SignalError {}

/// Runtime owner of a signal descriptor.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalOwner {
    /// Global signal not attached to a runtime instance.
    Global,
    /// Signal owned by a runtime instance.
    Instance(InstanceId),
}

impl fmt::Display for SignalOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global => f.write_str("global scope"),
            Self::Instance(id) => write!(f, "instance {id}"),
        }
    }
}

/// Signal delivery flush domain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalFlushDomain {
    /// Delivered during the frame-level signal/event flush phase.
    Frame,
    /// Delivered during the fixed-step signal/event flush phase.
    FixedStep,
}

/// Runtime signal descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDescriptor {
    /// Runtime signal ID.
    pub id: SignalId,
    /// Signal owner.
    pub owner: SignalOwner,
    /// Author-facing signal name.
    pub name: String,
    /// Frame phase where events for this signal are flushed.
    pub flush_domain: SignalFlushDomain,
}

/// Typed signal connection handle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalConnectionId(NonZeroU64);

impl SignalConnectionId {
    /// Creates a connection ID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        match NonZeroU64::new(raw) {
            Some(raw) => Self(raw),
            None => panic!("SignalConnectionId raw value must be non-zero"),
        }
    }

    /// Returns the raw numeric value for diagnostics/debugging.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for SignalConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignalConnectionId({})", self.raw())
    }
}

/// Signal connection lifecycle state.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignalConnectionState {
    /// Connection participates in future delivery.
    Connected,
    /// Connection was explicitly disconnected.
    Disconnected,
    /// Connection was invalidated by signal, owner, or world teardown.
    Invalidated,
}

/// Runtime signal connection record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalConnection {
    /// Typed connection handle.
    pub id: SignalConnectionId,
    /// Target signal.
    pub signal_id: SignalId,
    /// Deterministic creation order.
    pub order: u64,
    /// Current lifecycle state.
    pub state: SignalConnectionState,
}

/// Deterministic signal registry and connection table.
#[derive(Debug, Default)]
pub struct SignalRegistry {
    signals: Vec<SignalDescriptor>,
    connections: Vec<SignalConnection>,
}

impl SignalRegistry {
    /// Registers a global frame-level signal and returns its ID.
    ///
    /// # Errors
    ///
    /// Returns [`SignalError`] when the name is invalid or duplicated for the
    /// global owner.
    pub fn register(&mut self, name: impl Into<String>) -> SignalResult<SignalId> {
        self.register_with_owner(SignalOwner::Global, name, SignalFlushDomain::Frame)
    }

    /// Registers a signal for an owner and flush domain.
    ///
    /// # Errors
    ///
    /// Returns [`SignalError`] when the name is invalid or duplicated for the
    /// same owner.
    pub fn register_with_owner(
        &mut self,
        owner: SignalOwner,
        name: impl Into<String>,
        flush_domain: SignalFlushDomain,
    ) -> SignalResult<SignalId> {
        let name = name.into();
        validate_signal_name(&name)?;
        if self
            .signals
            .iter()
            .any(|signal| signal.owner == owner && signal.name == name)
        {
            return Err(SignalError::DuplicateSignal { owner, name });
        }

        let id = SignalId::new(self.signals.len() as u64 + 1);
        self.signals.push(SignalDescriptor {
            id,
            owner,
            name,
            flush_domain,
        });
        Ok(id)
    }

    /// Returns registered signals in deterministic registration order.
    #[must_use]
    pub fn signals(&self) -> &[SignalDescriptor] {
        &self.signals
    }

    /// Returns a signal descriptor by ID.
    #[must_use]
    pub fn signal(&self, id: SignalId) -> Option<&SignalDescriptor> {
        self.signals.iter().find(|signal| signal.id == id)
    }

    /// Connects to a registered signal.
    ///
    /// # Errors
    ///
    /// Returns [`SignalError::UnknownSignal`] when `signal_id` is not registered.
    pub fn connect(&mut self, signal_id: SignalId) -> SignalResult<SignalConnectionId> {
        if self.signal(signal_id).is_none() {
            return Err(SignalError::UnknownSignal { id: signal_id });
        }

        let id = SignalConnectionId::new(self.connections.len() as u64 + 1);
        let order = id.raw();
        self.connections.push(SignalConnection {
            id,
            signal_id,
            order,
            state: SignalConnectionState::Connected,
        });
        Ok(id)
    }

    /// Disconnects a connection. Disconnecting an already disconnected
    /// connection is allowed and leaves it disconnected.
    ///
    /// # Errors
    ///
    /// Returns [`SignalError::UnknownConnection`] when `connection_id` is not registered.
    pub fn disconnect(
        &mut self,
        connection_id: SignalConnectionId,
    ) -> SignalResult<SignalConnectionState> {
        let connection = self
            .connection_mut(connection_id)
            .ok_or(SignalError::UnknownConnection { id: connection_id })?;
        if connection.state == SignalConnectionState::Connected {
            connection.state = SignalConnectionState::Disconnected;
        }
        Ok(connection.state)
    }

    /// Invalidates every connection for `signal_id`.
    ///
    /// # Errors
    ///
    /// Returns [`SignalError::UnknownSignal`] when `signal_id` is not registered.
    pub fn invalidate_connections_for_signal(
        &mut self,
        signal_id: SignalId,
    ) -> SignalResult<usize> {
        if self.signal(signal_id).is_none() {
            return Err(SignalError::UnknownSignal { id: signal_id });
        }
        let mut invalidated = 0;
        for connection in &mut self.connections {
            if connection.signal_id == signal_id
                && connection.state != SignalConnectionState::Invalidated
            {
                connection.state = SignalConnectionState::Invalidated;
                invalidated += 1;
            }
        }
        Ok(invalidated)
    }

    /// Returns connections in deterministic creation order.
    #[must_use]
    pub fn connections(&self) -> &[SignalConnection] {
        &self.connections
    }

    /// Returns a connection by ID.
    #[must_use]
    pub fn connection(&self, id: SignalConnectionId) -> Option<&SignalConnection> {
        self.connections
            .iter()
            .find(|connection| connection.id == id)
    }

    fn connection_mut(&mut self, id: SignalConnectionId) -> Option<&mut SignalConnection> {
        self.connections
            .iter_mut()
            .find(|connection| connection.id == id)
    }
}

fn validate_signal_name(name: &str) -> SignalResult<()> {
    if name.is_empty() {
        return Err(SignalError::EmptySignalName);
    }
    if name
        .chars()
        .all(|character| character.is_ascii_alphanumeric())
        && name
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
    {
        return Ok(());
    }
    Err(SignalError::InvalidSignalName {
        name: name.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_signals_in_deterministic_order() {
        let mut registry = SignalRegistry::default();
        let touched = registry.register("Touched").unwrap();
        let stepped = registry
            .register_with_owner(
                SignalOwner::Instance(InstanceId::new(7)),
                "PhysicsTouched",
                SignalFlushDomain::FixedStep,
            )
            .unwrap();

        assert_eq!(touched, SignalId::new(1));
        assert_eq!(stepped, SignalId::new(2));
        assert_eq!(
            registry.signals(),
            &[
                SignalDescriptor {
                    id: touched,
                    owner: SignalOwner::Global,
                    name: "Touched".to_owned(),
                    flush_domain: SignalFlushDomain::Frame,
                },
                SignalDescriptor {
                    id: stepped,
                    owner: SignalOwner::Instance(InstanceId::new(7)),
                    name: "PhysicsTouched".to_owned(),
                    flush_domain: SignalFlushDomain::FixedStep,
                },
            ]
        );
    }

    #[test]
    fn register_rejects_invalid_and_duplicate_names_per_owner() {
        let mut registry = SignalRegistry::default();

        assert_eq!(
            registry.register("").unwrap_err(),
            SignalError::EmptySignalName
        );
        assert_eq!(
            registry.register("touched").unwrap_err(),
            SignalError::InvalidSignalName {
                name: "touched".to_owned()
            }
        );

        registry.register("Touched").unwrap();
        assert_eq!(
            registry.register("Touched").unwrap_err(),
            SignalError::DuplicateSignal {
                owner: SignalOwner::Global,
                name: "Touched".to_owned()
            }
        );

        registry
            .register_with_owner(
                SignalOwner::Instance(InstanceId::new(1)),
                "Touched",
                SignalFlushDomain::Frame,
            )
            .unwrap();
    }

    #[test]
    fn connections_use_deterministic_creation_order() {
        let mut registry = SignalRegistry::default();
        let touched = registry.register("Touched").unwrap();
        let changed = registry.register("Changed").unwrap();

        let first = registry.connect(touched).unwrap();
        let second = registry.connect(touched).unwrap();
        let third = registry.connect(changed).unwrap();

        assert_eq!(first, SignalConnectionId::new(1));
        assert_eq!(second, SignalConnectionId::new(2));
        assert_eq!(third, SignalConnectionId::new(3));
        assert_eq!(
            registry
                .connections()
                .iter()
                .map(|connection| connection.order)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn connect_rejects_unknown_signal() {
        let mut registry = SignalRegistry::default();

        assert_eq!(
            registry.connect(SignalId::new(99)).unwrap_err(),
            SignalError::UnknownSignal {
                id: SignalId::new(99)
            }
        );
    }

    #[test]
    fn disconnect_is_idempotent() {
        let mut registry = SignalRegistry::default();
        let touched = registry.register("Touched").unwrap();
        let connection = registry.connect(touched).unwrap();

        assert_eq!(
            registry.disconnect(connection).unwrap(),
            SignalConnectionState::Disconnected
        );
        assert_eq!(
            registry.disconnect(connection).unwrap(),
            SignalConnectionState::Disconnected
        );
        assert_eq!(
            registry.connection(connection).unwrap().state,
            SignalConnectionState::Disconnected
        );
    }

    #[test]
    fn disconnect_rejects_unknown_connection() {
        let mut registry = SignalRegistry::default();

        assert_eq!(
            registry
                .disconnect(SignalConnectionId::new(99))
                .unwrap_err(),
            SignalError::UnknownConnection {
                id: SignalConnectionId::new(99)
            }
        );
    }

    #[test]
    fn invalidates_connections_for_signal() {
        let mut registry = SignalRegistry::default();
        let touched = registry.register("Touched").unwrap();
        let changed = registry.register("Changed").unwrap();
        let first = registry.connect(touched).unwrap();
        let second = registry.connect(touched).unwrap();
        let other = registry.connect(changed).unwrap();

        assert_eq!(
            registry.invalidate_connections_for_signal(touched).unwrap(),
            2
        );
        assert_eq!(
            registry.connection(first).unwrap().state,
            SignalConnectionState::Invalidated
        );
        assert_eq!(
            registry.connection(second).unwrap().state,
            SignalConnectionState::Invalidated
        );
        assert_eq!(
            registry.connection(other).unwrap().state,
            SignalConnectionState::Connected
        );
    }
}
