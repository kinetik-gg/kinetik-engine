//! Deterministic signal bus contracts for Kinetik.

mod cleanup;
mod connection;
mod descriptor;
mod error;
mod event;
mod registry;

pub use cleanup::SignalOwnerCleanup;
pub use connection::{SignalConnection, SignalConnectionId, SignalConnectionState};
pub use descriptor::{SignalDescriptor, SignalFlushDomain, SignalOwner};
pub use error::{SignalError, SignalResult};
pub use event::{QueuedSignalEvent, SignalDeliveryRecord};
pub use registry::SignalRegistry;

#[cfg(test)]
mod tests;
