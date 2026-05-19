//! Runtime-agnostic scripting contracts for Kinetik.

use kinetik_core::{InstanceId, KinetikResult};

/// Script lifecycle entry points used by concrete script runtimes.
pub trait ScriptRuntime {
    /// Calls an instance script's ready hook.
    ///
    /// # Errors
    ///
    /// Returns an error when the script runtime cannot resolve the instance or the script hook fails.
    fn call_ready(&mut self, instance: InstanceId) -> KinetikResult<()>;

    /// Calls an instance script's frame update hook.
    ///
    /// # Errors
    ///
    /// Returns an error when the script runtime cannot resolve the instance or the script hook fails.
    fn call_update(&mut self, instance: InstanceId, delta_seconds: f32) -> KinetikResult<()>;

    /// Calls an instance script's fixed physics update hook.
    ///
    /// # Errors
    ///
    /// Returns an error when the script runtime cannot resolve the instance or the script hook fails.
    fn call_physics_update(
        &mut self,
        instance: InstanceId,
        fixed_delta_seconds: f32,
    ) -> KinetikResult<()>;
}
