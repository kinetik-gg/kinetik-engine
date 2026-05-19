//! Core primitives for Kinetik Engine.
//!
//! This crate contains small shared foundation types used by higher-level crates.

use core::fmt;

/// Standard result type used by Kinetik crates.
pub type KinetikResult<T> = Result<T, KinetikError>;

/// Foundational engine error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KinetikError {
    /// A handle or ID was not valid in the receiving system.
    InvalidHandle {
        /// Human-readable handle kind, such as `InstanceId`.
        kind: &'static str,
        /// Raw handle value that failed validation.
        id: u64,
    },
    /// A requested item was not found.
    NotFound {
        /// Human-readable item kind, such as `Resource`.
        kind: &'static str,
        /// Requested item name or path.
        name: String,
    },
    /// The operation is not implemented yet.
    NotImplemented {
        /// Feature name that is not implemented yet.
        feature: &'static str,
    },
}

impl fmt::Display for KinetikError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle { kind, id } => write!(f, "invalid {kind} handle: {id}"),
            Self::NotFound { kind, name } => write!(f, "{kind} not found: {name}"),
            Self::NotImplemented { feature } => write!(f, "feature not implemented: {feature}"),
        }
    }
}

impl std::error::Error for KinetikError {}

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            /// Creates a new typed ID from a raw value.
            #[must_use]
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            /// Returns the raw numeric value for serialization/debugging.
            #[must_use]
            pub const fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

typed_id!(/// Runtime instance ID.
InstanceId);
typed_id!(/// Stable serialized instance GUID surrogate until UUID support lands.
InstanceGuid);
typed_id!(/// Runtime resource ID.
ResourceId);
typed_id!(/// Runtime signal ID.
SignalId);
typed_id!(/// Runtime script ID.
ScriptId);
typed_id!(/// Runtime bundle ID.
BundleId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_ids_do_not_share_types() {
        let instance = InstanceId::new(7);
        let resource = ResourceId::new(7);
        assert_eq!(instance.raw(), resource.raw());
    }
}
