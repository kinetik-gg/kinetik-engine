//! Runtime app loop and world orchestration contracts for Kinetik.

use core::{fmt, num::NonZeroU64};

use kinetik_core::InstanceGuid;
use kinetik_scene::{Scene, SceneError, SceneInstanceDocument};

/// Runtime world ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeWorldId(NonZeroU64);

impl RuntimeWorldId {
    /// Creates a runtime world ID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        let Some(raw) = NonZeroU64::new(raw) else {
            panic!("RuntimeWorldId raw value must be non-zero");
        };
        Self(raw)
    }

    /// Returns the raw non-zero ID value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for RuntimeWorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeWorldId({})", self.raw())
    }
}

/// Runtime instance ID owned by a runtime world.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeInstanceId(NonZeroU64);

impl RuntimeInstanceId {
    /// Creates a runtime instance ID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        let Some(raw) = NonZeroU64::new(raw) else {
            panic!("RuntimeInstanceId raw value must be non-zero");
        };
        Self(raw)
    }

    /// Returns the raw non-zero ID value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for RuntimeInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeInstanceId({})", self.raw())
    }
}

/// Runtime instance cloned from edit scene state or spawned during play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInstanceRecord {
    /// Runtime-only instance ID.
    pub id: RuntimeInstanceId,
    /// Stable edit GUID when this runtime instance came from saved edit state.
    pub edit_guid: Option<InstanceGuid>,
    /// Registered class name cloned from edit state.
    pub class_name: String,
    /// Runtime display name cloned from edit state.
    pub name: String,
    /// Runtime parent ID.
    pub parent: Option<RuntimeInstanceId>,
    /// Ordered runtime child IDs.
    pub children: Vec<RuntimeInstanceId>,
}

/// Sandboxed runtime world derived from edit scene state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorld {
    id: RuntimeWorldId,
    instances: Vec<RuntimeInstanceRecord>,
    root: Option<RuntimeInstanceId>,
    next_instance_id: u64,
}

impl RuntimeWorld {
    /// Creates a runtime world clone from an edit scene.
    ///
    /// Runtime IDs are owned by the runtime world and edit GUIDs are retained
    /// only as provenance for saved instances.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the edit scene cannot be exported as a scene
    /// document, such as when it has no root.
    pub fn clone_from_edit_scene(id: RuntimeWorldId, scene: &Scene) -> Result<Self, SceneError> {
        let document = scene.to_document()?;
        let mut world = Self {
            id,
            instances: Vec::new(),
            root: None,
            next_instance_id: 1,
        };
        let root = world.clone_document_instance(document.root, None);
        world.root = Some(root);
        Ok(world)
    }

    /// Returns this runtime world's ID.
    #[must_use]
    pub const fn id(&self) -> RuntimeWorldId {
        self.id
    }

    /// Returns the root runtime instance ID when present.
    #[must_use]
    pub const fn root_id(&self) -> Option<RuntimeInstanceId> {
        self.root
    }

    /// Returns all runtime instances in deterministic parent-before-child order.
    #[must_use]
    pub fn instances(&self) -> &[RuntimeInstanceRecord] {
        &self.instances
    }

    /// Returns a runtime instance by runtime ID.
    #[must_use]
    pub fn get(&self, id: RuntimeInstanceId) -> Option<&RuntimeInstanceRecord> {
        self.instances.iter().find(|instance| instance.id == id)
    }

    /// Returns the runtime instance cloned from `edit_guid`, if any.
    #[must_use]
    pub fn runtime_id_for_edit_guid(&self, edit_guid: InstanceGuid) -> Option<RuntimeInstanceId> {
        self.instances
            .iter()
            .find(|instance| instance.edit_guid == Some(edit_guid))
            .map(|instance| instance.id)
    }

    fn clone_document_instance(
        &mut self,
        document: SceneInstanceDocument,
        parent: Option<RuntimeInstanceId>,
    ) -> RuntimeInstanceId {
        let id = self.next_runtime_instance_id();
        let children = document.children;
        let index = self.instances.len();
        self.instances.push(RuntimeInstanceRecord {
            id,
            edit_guid: Some(document.guid),
            class_name: document.class_name,
            name: document.name,
            parent,
            children: Vec::new(),
        });
        let child_ids = children
            .into_iter()
            .map(|child| self.clone_document_instance(child, Some(id)))
            .collect();
        self.instances[index].children = child_ids;
        id
    }

    fn next_runtime_instance_id(&mut self) -> RuntimeInstanceId {
        let id = RuntimeInstanceId::new(self.next_instance_id);
        self.next_instance_id += 1;
        id
    }
}

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

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-app"
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetik_scene::ROOT_CLASS_NAME;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "kinetik-app");
    }

    #[test]
    fn runtime_world_clone_preserves_edit_guid_mapping() {
        let scene = kinetik_scene::Scene::default_scene().unwrap();
        let document = scene.to_document().unwrap();
        let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();

        assert_eq!(world.id(), RuntimeWorldId::new(1));
        assert_eq!(world.root_id(), Some(RuntimeInstanceId::new(1)));
        assert_eq!(
            world.runtime_id_for_edit_guid(document.root.guid),
            Some(RuntimeInstanceId::new(1))
        );
        assert_eq!(
            world.get(RuntimeInstanceId::new(1)).unwrap().class_name,
            ROOT_CLASS_NAME
        );
        assert_eq!(
            world.get(RuntimeInstanceId::new(2)).unwrap().parent,
            Some(RuntimeInstanceId::new(1))
        );
    }

    #[test]
    fn runtime_world_clone_uses_deterministic_parent_before_child_order() {
        let scene = kinetik_scene::Scene::default_scene().unwrap();
        let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(7), &scene).unwrap();
        let names: Vec<&str> = world
            .instances()
            .iter()
            .map(|instance| instance.name.as_str())
            .collect();

        assert_eq!(
            names,
            vec![
                "Game",
                "Workspace",
                "Prefabs",
                "Scripts",
                "UI",
                "Lighting",
                "Audio",
                "Physics",
                "Assets",
                "Packages",
            ]
        );
        assert_eq!(
            world.get(RuntimeInstanceId::new(1)).unwrap().children,
            (2..=10).map(RuntimeInstanceId::new).collect::<Vec<_>>()
        );
    }

    #[test]
    fn runtime_world_clone_does_not_require_edit_instance_ids() {
        let mut scene = kinetik_scene::Scene::new();
        let root = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let node = scene.add_child(root, "Node3D", "Node").unwrap();
        let edit_root_raw = root.raw();
        let edit_node_raw = node.raw();

        let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(3), &scene).unwrap();

        assert_eq!(world.root_id().unwrap().raw(), 1);
        assert_eq!(world.get(RuntimeInstanceId::new(2)).unwrap().name, "Node");
        assert_eq!(edit_root_raw, 1);
        assert_eq!(edit_node_raw, 2);
        assert_ne!(
            core::any::type_name::<RuntimeInstanceId>(),
            core::any::type_name::<kinetik_core::InstanceId>()
        );
    }

    #[test]
    fn runtime_world_clone_requires_edit_scene_root() {
        let scene = kinetik_scene::Scene::new();

        assert_eq!(
            RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap_err(),
            SceneError::MissingRoot
        );
    }

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
