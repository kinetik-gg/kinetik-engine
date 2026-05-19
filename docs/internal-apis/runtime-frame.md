# RuntimeWorld and FrameScheduler Contract

## Purpose

Define runtime world identity and frame scheduling before scripts, physics,
signals, rendering, play mode, diagnostics, and MCP runtime inspection depend on
runtime behavior.

## Owning Crates

- Future runtime owner crate or `kinetik-app`: runtime world lifecycle,
  subsystem orchestration, frame stepping, and runtime diagnostics/log
  attribution.
- `kinetik-scene`: edit scene data used to create runtime worlds.
- `kinetik-signal`, `kinetik-script`, `kinetik-physics`, `kinetik-render`:
  participate in frame phases through runtime-owned scheduling.
- `kinetik-editor`: starts/stops/steps play worlds but does not own runtime
  internals.

## RuntimeWorld Contract

`RuntimeWorld` is a sandbox derived from edit state:

- Runtime IDs are distinct from edit IDs.
- Saved instances cloned from edit state keep GUID mapping where available.
- Runtime-only spawns have runtime identity but no saved GUID unless explicitly
  persisted by an editor command.
- Stopping play destroys the runtime world.
- Applying play changes back to edit mode requires command/change-record flow.

Runtime world state must not mutate saved project state directly.

## FrameScheduler Contract

The first scheduler is deterministic and single-threaded.

Required phases:

1. Poll platform/input.
2. Begin frame diagnostics/log scope.
3. Apply queued structural changes from prior safe points.
4. Run variable script `Update(dt)`.
5. Run zero or more fixed simulation steps:
   - run `PhysicsUpdate(fixed_dt)`
   - step physics
   - collect collision/physics events
   - flush fixed-step signals/events
   - apply fixed-step safe structural changes
6. Flush frame-level signals/events.
7. Update derived transforms and world state.
8. Update animation and audio.
9. Render from a coherent world snapshot.
10. End frame cleanup.

Exact micro-order can evolve only if it preserves these synchronization
principles.

## Dependency Boundaries

- Runtime/app dependencies remain dependency-light per
  `docs/dependency-proposals/runtime-app.md`.
- Physics, rendering, and Luau dependencies stay behind their approved domain
  boundaries.
- Runtime public contracts expose Kinetik-owned world, frame, diagnostic, and
  handle types.

## Serialized-Format Impact

No serialized-format impact is approved.

Runtime-only state is not serialized into project source files unless an
explicit apply-back command defines a saved edit-state mutation.

## Diagnostics Behavior

Runtime diagnostics and logs should be attributable to:

- Runtime world ID.
- Frame index.
- Fixed-step index when applicable.
- Source subsystem.
- Script asset and owning instance when applicable.
- Edit GUID mapping when runtime state derives from saved state.

## Public API Constraints

- Standalone runtime must not depend on editor crates.
- MCP runtime inspection goes through editor-owned MCP surfaces, not direct MCP
  dependencies in runtime crates.
- Runtime internals may use ECS-like structures only as implementation details.

## Follow-Up Issues

- M8 runtime world clone.
- M8 runtime identity mapping.
- M8 frame step skeleton.
- M8 fixed-step scheduler.
- M22 play mode control slice.
