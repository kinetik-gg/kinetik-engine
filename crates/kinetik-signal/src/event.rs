use kinetik_core::{InstanceGuid, InstanceId, SignalId};

use crate::{SignalConnectionId, SignalFlushDomain};

/// Queued signal event metadata.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueuedSignalEvent {
    /// Target signal.
    pub signal_id: SignalId,
    /// Signal flush domain.
    pub flush_domain: SignalFlushDomain,
    /// Emit sequence within the flush domain.
    pub sequence: u64,
    /// Runtime frame index when known.
    pub frame_index: u64,
    /// Fixed-step index for fixed-step events.
    pub fixed_step_index: Option<u64>,
    /// Runtime emitter instance when known.
    pub emitter: Option<InstanceId>,
    /// Edit GUID mapping for the emitter when available.
    pub edit_guid: Option<InstanceGuid>,
}

/// A queued event paired with a connected subscriber.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SignalDeliveryRecord {
    /// Event to deliver.
    pub event: QueuedSignalEvent,
    /// Connection receiving the event.
    pub connection_id: SignalConnectionId,
    /// Connection creation order used as the delivery tie-breaker.
    pub connection_order: u64,
}
