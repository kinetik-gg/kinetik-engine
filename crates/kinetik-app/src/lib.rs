//! Runtime app loop and world orchestration contracts for Kinetik.

mod frame;
mod runtime_world;

pub use frame::{FramePhase, FrameScheduler, FrameStepRecord, FrameStepResult};
pub use runtime_world::{
    RuntimeInstanceId, RuntimeInstanceRecord, RuntimeWorld, RuntimeWorldError, RuntimeWorldId,
    RuntimeWorldResult,
};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-app"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "kinetik-app");
    }
}
