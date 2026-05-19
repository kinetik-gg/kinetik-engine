use core::fmt;

use kinetik_core::{InstanceId, SignalId};

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
