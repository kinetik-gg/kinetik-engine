use core::fmt;

use kinetik_core::{InstanceId, ResourceId};

/// Error returned when a script-facing safe handle cannot be resolved.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScriptHandleError {
    /// The handle was explicitly invalidated by teardown or a safe sync point.
    Invalidated,
}

impl fmt::Display for ScriptHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalidated => f.write_str("script handle was invalidated"),
        }
    }
}

/// Safe script-facing handle to a runtime instance.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScriptInstanceHandle {
    instance_id: InstanceId,
    valid: bool,
}

impl ScriptInstanceHandle {
    /// Creates a valid instance handle.
    #[must_use]
    pub const fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            valid: true,
        }
    }

    /// Creates an invalidated handle for tests and teardown bookkeeping.
    #[must_use]
    pub const fn invalidated(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            valid: false,
        }
    }

    /// Returns the runtime instance ID when the handle is valid.
    ///
    /// # Errors
    ///
    /// Returns [`ScriptHandleError::Invalidated`] when the handle has been
    /// invalidated by runtime teardown or a safe structural sync point.
    pub const fn resolve(self) -> Result<InstanceId, ScriptHandleError> {
        if self.valid {
            Ok(self.instance_id)
        } else {
            Err(ScriptHandleError::Invalidated)
        }
    }
}

/// Safe script-facing handle to a loaded resource.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceScriptHandle {
    resource_id: ResourceId,
    valid: bool,
}

impl ResourceScriptHandle {
    /// Creates a valid resource handle.
    #[must_use]
    pub const fn new(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            valid: true,
        }
    }

    /// Creates an invalidated handle for tests and teardown bookkeeping.
    #[must_use]
    pub const fn invalidated(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            valid: false,
        }
    }

    /// Returns the runtime resource ID when the handle is valid.
    ///
    /// # Errors
    ///
    /// Returns [`ScriptHandleError::Invalidated`] when the handle has been
    /// invalidated by resource unload or runtime teardown.
    pub const fn resolve(self) -> Result<ResourceId, ScriptHandleError> {
        if self.valid {
            Ok(self.resource_id)
        } else {
            Err(ScriptHandleError::Invalidated)
        }
    }
}
