/// Runtime frame phases in deterministic execution order.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FramePhase {
    /// Poll platform and input state.
    PollPlatformInput,
    /// Begin frame diagnostics and log attribution scope.
    BeginFrameDiagnostics,
    /// Apply structural changes queued before this frame.
    ApplyQueuedStructuralChanges,
    /// Run variable-rate update.
    VariableUpdate,
    /// Run fixed-rate simulation update.
    FixedUpdate,
    /// Step the physics world.
    StepPhysics,
    /// Collect physics events produced by the physics step.
    CollectPhysicsEvents,
    /// Flush fixed-step signals and events.
    FlushFixedSignals,
    /// Apply structural changes queued during the fixed step.
    ApplyFixedStructuralChanges,
    /// Flush frame-level signals and events.
    FlushFrameSignals,
    /// Update derived transforms and world state.
    UpdateDerivedWorldState,
    /// Update animation and audio systems.
    UpdateAnimationAndAudio,
    /// Render from a coherent world snapshot.
    RenderSnapshot,
    /// End frame cleanup.
    EndFrameCleanup,
}

/// Ordered record for a runtime frame phase.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameStepRecord {
    /// Phase executed by the scheduler.
    pub phase: FramePhase,
    /// Runtime frame index for this phase.
    pub frame_index: u64,
    /// Fixed-step index when this phase belongs to fixed simulation.
    pub fixed_step_index: Option<u64>,
}

/// Result of one runtime scheduler frame step.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameStepResult {
    /// Runtime frame index for this step.
    pub frame_index: u64,
    /// Variable frame delta passed to the scheduler.
    pub delta_seconds: f32,
    /// Number of fixed simulation steps emitted this frame.
    pub fixed_steps: usize,
    /// Fixed-step accumulator remaining after this frame.
    pub accumulator_seconds: f32,
    /// Ordered phase records emitted for this frame.
    pub records: Vec<FrameStepRecord>,
}

/// Deterministic single-threaded frame scheduler.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FrameScheduler {
    frame_index: u64,
    next_fixed_step_index: u64,
    fixed_delta_seconds: f32,
    accumulator_seconds: f32,
}

impl FrameScheduler {
    /// Creates a frame scheduler with a fixed simulation timestep.
    ///
    /// # Panics
    ///
    /// Panics when `fixed_delta_seconds` is not positive.
    #[must_use]
    pub fn new(fixed_delta_seconds: f32) -> Self {
        assert!(
            fixed_delta_seconds > 0.0,
            "fixed delta seconds must be positive"
        );
        Self {
            frame_index: 0,
            next_fixed_step_index: 1,
            fixed_delta_seconds,
            accumulator_seconds: 0.0,
        }
    }

    /// Returns the most recently completed frame index.
    #[must_use]
    pub const fn frame_index(&self) -> u64 {
        self.frame_index
    }

    /// Returns the fixed-step index that will be assigned next.
    #[must_use]
    pub const fn next_fixed_step_index(&self) -> u64 {
        self.next_fixed_step_index
    }

    /// Returns the fixed simulation timestep in seconds.
    #[must_use]
    pub const fn fixed_delta_seconds(&self) -> f32 {
        self.fixed_delta_seconds
    }

    /// Returns the current fixed-step accumulator in seconds.
    #[must_use]
    pub const fn accumulator_seconds(&self) -> f32 {
        self.accumulator_seconds
    }

    /// Steps one runtime frame and returns the deterministic phase records.
    ///
    /// # Panics
    ///
    /// Panics when `delta_seconds` is negative.
    #[must_use]
    pub fn step(&mut self, delta_seconds: f32) -> FrameStepResult {
        assert!(delta_seconds >= 0.0, "delta seconds must be non-negative");

        self.frame_index += 1;
        self.accumulator_seconds += delta_seconds;

        let frame_index = self.frame_index;
        let mut records = Vec::new();
        push_frame_phase_records(&mut records, frame_index, FRAME_START_PHASES);

        let mut fixed_steps = 0;
        while self.accumulator_seconds >= self.fixed_delta_seconds {
            let fixed_step_index = self.next_fixed_step_index;
            self.next_fixed_step_index += 1;
            fixed_steps += 1;
            self.accumulator_seconds -= self.fixed_delta_seconds;
            push_fixed_phase_records(&mut records, frame_index, fixed_step_index);
        }

        push_frame_phase_records(&mut records, frame_index, FRAME_END_PHASES);

        FrameStepResult {
            frame_index,
            delta_seconds,
            fixed_steps,
            accumulator_seconds: self.accumulator_seconds,
            records,
        }
    }
}

const FRAME_START_PHASES: &[FramePhase] = &[
    FramePhase::PollPlatformInput,
    FramePhase::BeginFrameDiagnostics,
    FramePhase::ApplyQueuedStructuralChanges,
    FramePhase::VariableUpdate,
];

const FIXED_STEP_PHASES: &[FramePhase] = &[
    FramePhase::FixedUpdate,
    FramePhase::StepPhysics,
    FramePhase::CollectPhysicsEvents,
    FramePhase::FlushFixedSignals,
    FramePhase::ApplyFixedStructuralChanges,
];

const FRAME_END_PHASES: &[FramePhase] = &[
    FramePhase::FlushFrameSignals,
    FramePhase::UpdateDerivedWorldState,
    FramePhase::UpdateAnimationAndAudio,
    FramePhase::RenderSnapshot,
    FramePhase::EndFrameCleanup,
];

fn push_frame_phase_records(
    records: &mut Vec<FrameStepRecord>,
    frame_index: u64,
    phases: &[FramePhase],
) {
    records.extend(phases.iter().map(|phase| FrameStepRecord {
        phase: *phase,
        frame_index,
        fixed_step_index: None,
    }));
}

fn push_fixed_phase_records(
    records: &mut Vec<FrameStepRecord>,
    frame_index: u64,
    fixed_step_index: u64,
) {
    records.extend(FIXED_STEP_PHASES.iter().map(|phase| FrameStepRecord {
        phase: *phase,
        frame_index,
        fixed_step_index: Some(fixed_step_index),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_scheduler_emits_adr_order_without_fixed_step() {
        let mut scheduler = FrameScheduler::new(1.0 / 60.0);

        let result = scheduler.step(0.01);

        assert_eq!(result.frame_index, 1);
        assert_eq!(result.fixed_steps, 0);
        assert_close(result.accumulator_seconds, 0.01);
        assert_eq!(
            phases(&result),
            vec![
                FramePhase::PollPlatformInput,
                FramePhase::BeginFrameDiagnostics,
                FramePhase::ApplyQueuedStructuralChanges,
                FramePhase::VariableUpdate,
                FramePhase::FlushFrameSignals,
                FramePhase::UpdateDerivedWorldState,
                FramePhase::UpdateAnimationAndAudio,
                FramePhase::RenderSnapshot,
                FramePhase::EndFrameCleanup,
            ]
        );
        assert!(result
            .records
            .iter()
            .all(|record| record.frame_index == 1 && record.fixed_step_index.is_none()));
    }

    #[test]
    fn frame_scheduler_emits_fixed_steps_between_variable_and_frame_flush() {
        let mut scheduler = FrameScheduler::new(0.5);

        let result = scheduler.step(1.1);

        assert_eq!(result.fixed_steps, 2);
        assert_close(result.accumulator_seconds, 0.1);
        assert_eq!(
            phases(&result),
            vec![
                FramePhase::PollPlatformInput,
                FramePhase::BeginFrameDiagnostics,
                FramePhase::ApplyQueuedStructuralChanges,
                FramePhase::VariableUpdate,
                FramePhase::FixedUpdate,
                FramePhase::StepPhysics,
                FramePhase::CollectPhysicsEvents,
                FramePhase::FlushFixedSignals,
                FramePhase::ApplyFixedStructuralChanges,
                FramePhase::FixedUpdate,
                FramePhase::StepPhysics,
                FramePhase::CollectPhysicsEvents,
                FramePhase::FlushFixedSignals,
                FramePhase::ApplyFixedStructuralChanges,
                FramePhase::FlushFrameSignals,
                FramePhase::UpdateDerivedWorldState,
                FramePhase::UpdateAnimationAndAudio,
                FramePhase::RenderSnapshot,
                FramePhase::EndFrameCleanup,
            ]
        );
        assert_eq!(
            fixed_step_indices(&result),
            vec![1, 1, 1, 1, 1, 2, 2, 2, 2, 2]
        );
    }

    #[test]
    fn frame_scheduler_carries_accumulator_and_indices_deterministically() {
        let mut scheduler = FrameScheduler::new(0.25);

        let first = scheduler.step(0.1);
        let second = scheduler.step(0.2);
        let third = scheduler.step(0.45);

        assert_eq!(first.frame_index, 1);
        assert_eq!(first.fixed_steps, 0);
        assert_close(first.accumulator_seconds, 0.1);
        assert_eq!(second.frame_index, 2);
        assert_eq!(second.fixed_steps, 1);
        assert_close(second.accumulator_seconds, 0.05);
        assert_eq!(third.frame_index, 3);
        assert_eq!(third.fixed_steps, 2);
        assert_close(third.accumulator_seconds, 0.0);
        assert_eq!(scheduler.frame_index(), 3);
        assert_eq!(scheduler.next_fixed_step_index(), 4);
    }

    #[test]
    fn frame_scheduler_rejects_invalid_times() {
        assert!(std::panic::catch_unwind(|| FrameScheduler::new(0.0)).is_err());

        let mut scheduler = FrameScheduler::new(0.25);
        assert!(std::panic::catch_unwind(move || scheduler.step(-0.1)).is_err());
    }

    fn phases(result: &FrameStepResult) -> Vec<FramePhase> {
        result.records.iter().map(|record| record.phase).collect()
    }

    fn fixed_step_indices(result: &FrameStepResult) -> Vec<u64> {
        result
            .records
            .iter()
            .filter_map(|record| record.fixed_step_index)
            .collect()
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= f32::EPSILON * 16.0,
            "expected {actual} to be close to {expected}"
        );
    }
}
