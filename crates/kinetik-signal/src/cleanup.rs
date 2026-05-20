use kinetik_core::SignalId;

use crate::{QueuedSignalEvent, SignalConnectionId, SignalOwner};

/// Summary of signal state removed or invalidated for an owner teardown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalOwnerCleanup {
    /// Owner whose runtime signal state was cleaned up.
    pub owner: SignalOwner,
    /// Signal descriptors removed for the owner in registration order.
    pub affected_signals: Vec<SignalId>,
    /// Connections invalidated because their target signal was removed.
    pub invalidated_connections: Vec<SignalConnectionId>,
    /// Queued events removed because they target or were emitted by the owner.
    pub removed_events: Vec<QueuedSignalEvent>,
}
