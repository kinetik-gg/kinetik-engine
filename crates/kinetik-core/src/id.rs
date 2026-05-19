use core::{fmt, num::NonZeroU64};

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(NonZeroU64);

        impl $name {
            /// Creates a new typed ID from a non-zero raw value.
            ///
            /// Kinetik reserves zero as invalid for every typed ID kind,
            /// including runtime IDs and stable GUID surrogates.
            ///
            /// # Panics
            ///
            /// Panics when `raw` is zero.
            #[must_use]
            pub const fn new(raw: u64) -> Self {
                match NonZeroU64::new(raw) {
                    Some(raw) => Self(raw),
                    None => panic!(concat!(stringify!($name), " raw value must be non-zero")),
                }
            }

            /// Returns the raw numeric value for serialization/debugging.
            #[must_use]
            pub const fn raw(self) -> u64 {
                self.0.get()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.raw())
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
