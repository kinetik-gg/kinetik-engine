//! Deterministic signal bus contracts for Kinetik.

use kinetik_core::SignalId;

/// Minimal signal descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDescriptor {
    /// Runtime signal ID.
    pub id: SignalId,
    /// Author-facing signal name.
    pub name: String,
}

/// Minimal deterministic signal registry placeholder.
#[derive(Debug, Default)]
pub struct SignalRegistry {
    signals: Vec<SignalDescriptor>,
}

impl SignalRegistry {
    /// Registers a signal and returns its ID.
    pub fn register(&mut self, name: impl Into<String>) -> SignalId {
        let id = SignalId::new(self.signals.len() as u64 + 1);
        self.signals.push(SignalDescriptor {
            id,
            name: name.into(),
        });
        id
    }

    /// Returns registered signals in deterministic registration order.
    #[must_use]
    pub fn signals(&self) -> &[SignalDescriptor] {
        &self.signals
    }
}
