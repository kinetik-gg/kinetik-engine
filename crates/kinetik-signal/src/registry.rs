use kinetik_core::{InstanceGuid, InstanceId, SignalId};

use crate::{
    QueuedSignalEvent, SignalConnection, SignalConnectionId, SignalConnectionState,
    SignalDeliveryRecord, SignalDescriptor, SignalError, SignalFlushDomain, SignalOwner,
    SignalResult,
};

/// Deterministic signal registry and connection table.
#[derive(Debug, Default)]
pub struct SignalRegistry {
    signals: Vec<SignalDescriptor>,
    connections: Vec<SignalConnection>,
    events: Vec<QueuedSignalEvent>,
    next_frame_sequence: u64,
    next_fixed_step_sequence: u64,
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

    /// Queues a frame-level event.
    ///
    /// # Errors
    ///
    /// Returns [`SignalError`] when `signal_id` is not registered or does not
    /// use the frame-level flush domain.
    pub fn queue_frame_event(
        &mut self,
        signal_id: SignalId,
        frame_index: u64,
        emitter: Option<InstanceId>,
        edit_guid: Option<InstanceGuid>,
    ) -> SignalResult<QueuedSignalEvent> {
        self.queue_event(
            signal_id,
            SignalFlushDomain::Frame,
            frame_index,
            None,
            emitter,
            edit_guid,
        )
    }

    /// Queues a fixed-step event.
    ///
    /// # Errors
    ///
    /// Returns [`SignalError`] when `signal_id` is not registered or does not
    /// use the fixed-step flush domain.
    pub fn queue_fixed_step_event(
        &mut self,
        signal_id: SignalId,
        frame_index: u64,
        fixed_step_index: u64,
        emitter: Option<InstanceId>,
        edit_guid: Option<InstanceGuid>,
    ) -> SignalResult<QueuedSignalEvent> {
        self.queue_event(
            signal_id,
            SignalFlushDomain::FixedStep,
            frame_index,
            Some(fixed_step_index),
            emitter,
            edit_guid,
        )
    }

    /// Returns queued events in deterministic queue order.
    #[must_use]
    pub fn events(&self) -> &[QueuedSignalEvent] {
        &self.events
    }

    /// Returns delivery records for queued events in `flush_domain`.
    ///
    /// Records are ordered by event queue order and then connection creation
    /// order. Disconnected and invalidated connections are skipped.
    #[must_use]
    pub fn delivery_records(&self, flush_domain: SignalFlushDomain) -> Vec<SignalDeliveryRecord> {
        let mut records = Vec::new();
        for event in self
            .events
            .iter()
            .filter(|event| event.flush_domain == flush_domain)
        {
            for connection in self.connections.iter().filter(|connection| {
                connection.signal_id == event.signal_id
                    && connection.state == SignalConnectionState::Connected
            }) {
                records.push(SignalDeliveryRecord {
                    event: *event,
                    connection_id: connection.id,
                    connection_order: connection.order,
                });
            }
        }
        records
    }

    /// Drains queued events for `flush_domain` in deterministic order.
    pub fn drain_events(&mut self, flush_domain: SignalFlushDomain) -> Vec<QueuedSignalEvent> {
        let mut drained = Vec::new();
        let mut retained = Vec::new();
        for event in self.events.drain(..) {
            if event.flush_domain == flush_domain {
                drained.push(event);
            } else {
                retained.push(event);
            }
        }
        self.events = retained;
        drained
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

    fn queue_event(
        &mut self,
        signal_id: SignalId,
        flush_domain: SignalFlushDomain,
        frame_index: u64,
        fixed_step_index: Option<u64>,
        emitter: Option<InstanceId>,
        edit_guid: Option<InstanceGuid>,
    ) -> SignalResult<QueuedSignalEvent> {
        let signal = self
            .signal(signal_id)
            .ok_or(SignalError::UnknownSignal { id: signal_id })?;
        if signal.flush_domain != flush_domain {
            return Err(SignalError::WrongFlushDomain {
                id: signal_id,
                expected: signal.flush_domain,
                actual: flush_domain,
            });
        }

        let sequence = match flush_domain {
            SignalFlushDomain::Frame => {
                self.next_frame_sequence += 1;
                self.next_frame_sequence
            }
            SignalFlushDomain::FixedStep => {
                self.next_fixed_step_sequence += 1;
                self.next_fixed_step_sequence
            }
        };
        let event = QueuedSignalEvent {
            signal_id,
            flush_domain,
            sequence,
            frame_index,
            fixed_step_index,
            emitter,
            edit_guid,
        };
        self.events.push(event);
        Ok(event)
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
