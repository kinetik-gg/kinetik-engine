use core::{fmt, num::NonZeroU64};

use kinetik_core::SignalId;

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
