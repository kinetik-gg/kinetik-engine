use kinetik_core::KinetikResult;

use crate::ScriptAttachmentId;

/// Script lifecycle hook called by the frame scheduler.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LifecyclePhase {
    /// Initial hook after an attachment is resolved.
    Ready,
    /// Variable frame update.
    Update,
    /// Fixed-step simulation update.
    PhysicsUpdate,
    /// Teardown hook before a script attachment exits.
    Exit,
}

/// One scheduled lifecycle call.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LifecycleCall {
    /// Script attachment to invoke.
    pub attachment_id: ScriptAttachmentId,
    /// Lifecycle hook to call.
    pub phase: LifecyclePhase,
    /// Delta seconds for update phases.
    pub delta_seconds: Option<f32>,
}

impl LifecycleCall {
    /// Creates a lifecycle call without a delta.
    #[must_use]
    pub const fn new(attachment_id: ScriptAttachmentId, phase: LifecyclePhase) -> Self {
        Self {
            attachment_id,
            phase,
            delta_seconds: None,
        }
    }

    /// Creates a lifecycle call with a delta.
    #[must_use]
    pub const fn with_delta(
        attachment_id: ScriptAttachmentId,
        phase: LifecyclePhase,
        delta_seconds: f32,
    ) -> Self {
        Self {
            attachment_id,
            phase,
            delta_seconds: Some(delta_seconds),
        }
    }
}

/// Script lifecycle entry points implemented by concrete language runtimes.
pub trait ScriptRuntime {
    /// Calls an instance script hook.
    ///
    /// # Errors
    ///
    /// Returns an error when the script runtime cannot resolve the attachment
    /// or when the script hook fails.
    fn call_lifecycle(&mut self, call: LifecycleCall) -> KinetikResult<()>;
}

/// Runtime-owned host that dispatches lifecycle calls in scheduler order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScriptRuntimeHost {
    pending: Vec<LifecycleCall>,
}

impl ScriptRuntimeHost {
    /// Creates an empty script runtime host.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Queues a `Ready` hook.
    pub fn queue_ready(&mut self, attachment_id: ScriptAttachmentId) {
        self.pending
            .push(LifecycleCall::new(attachment_id, LifecyclePhase::Ready));
    }

    /// Queues an `Update` hook.
    pub fn queue_update(&mut self, attachment_id: ScriptAttachmentId, delta_seconds: f32) {
        self.pending.push(LifecycleCall::with_delta(
            attachment_id,
            LifecyclePhase::Update,
            delta_seconds,
        ));
    }

    /// Queues a `PhysicsUpdate` hook.
    pub fn queue_physics_update(&mut self, attachment_id: ScriptAttachmentId, fixed_delta: f32) {
        self.pending.push(LifecycleCall::with_delta(
            attachment_id,
            LifecyclePhase::PhysicsUpdate,
            fixed_delta,
        ));
    }

    /// Queues an `Exit` hook.
    pub fn queue_exit(&mut self, attachment_id: ScriptAttachmentId) {
        self.pending
            .push(LifecycleCall::new(attachment_id, LifecyclePhase::Exit));
    }

    /// Returns queued calls without dispatching them.
    #[must_use]
    pub fn pending_calls(&self) -> &[LifecycleCall] {
        &self.pending
    }

    /// Drains queued lifecycle calls through a concrete runtime in FIFO order.
    ///
    /// # Errors
    ///
    /// Returns the first error emitted by the concrete script runtime.
    pub fn dispatch<R: ScriptRuntime>(&mut self, runtime: &mut R) -> KinetikResult<()> {
        for call in self.pending.drain(..) {
            runtime.call_lifecycle(call)?;
        }

        Ok(())
    }
}

/// Test helper and fake-runtime log for lifecycle ordering assertions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LifecycleCallLog {
    calls: Vec<LifecycleCall>,
}

impl LifecycleCallLog {
    /// Creates an empty call log.
    #[must_use]
    pub const fn new() -> Self {
        Self { calls: Vec::new() }
    }

    /// Returns recorded calls.
    #[must_use]
    pub fn calls(&self) -> &[LifecycleCall] {
        &self.calls
    }
}

impl ScriptRuntime for LifecycleCallLog {
    fn call_lifecycle(&mut self, call: LifecycleCall) -> KinetikResult<()> {
        self.calls.push(call);
        Ok(())
    }
}
