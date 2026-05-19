# ADR 0019: Edit, Play, and Runtime State Boundaries

## Status

Accepted as initial project direction.

## Context

Game editors become hard to trust when play-mode changes leak into saved project
state, dirty state is unclear, runtime-spawned objects persist accidentally, or
automation targets the wrong world.

Kinetik needs a strict boundary between editable source state and live runtime
simulation.

## Decision

Kinetik separates edit-mode source state from play-mode runtime state.

Play mode operates on a sandboxed runtime copy. Changes only persist when
explicitly applied through editor commands.

## Worlds

Edit world:

- Serialized project, scene, prefab, and manifest state.
- Editor selection.
- Inspector state.
- Undo and redo.
- Dirty tracking.
- Diagnostics for saved project health.

Play world:

- Runtime clone or sandbox derived from the edit world.
- Script execution.
- Physics simulation.
- Runtime-spawned instances.
- Temporary gameplay state.
- Runtime diagnostics and logs.

## Rules

- Pressing Play creates a runtime world from the current edit scene.
- The play world receives runtime IDs distinct from edit-mode IDs.
- Stable GUIDs may map across worlds for instances cloned from saved state.
- Runtime-only spawned instances get runtime identity but no saved GUID unless
  explicitly persisted.
- Stopping Play destroys the play world.
- Applying play changes back to edit mode requires an explicit editor command.
- MCP mutating commands must declare or imply a clear target mode: edit or play.
- Ambiguous commands must fail with diagnostics instead of guessing.

## MCP Examples

```text
scene.set_property(mode = "edit", instance = "/Game/Lighting/Sun", property = "Intensity", value = 4)
play.set_property(mode = "play", instance = runtime_id, property = "Health", value = 10)
play.apply_change_to_edit(...)
```

## Consequences

- Play mode cannot silently corrupt saved project state.
- Runtime-spawned objects are temporary unless explicitly persisted.
- Humans and agents can tell whether they are editing the project or the live
  simulation.
- Dirty state remains tied to edit-world commands and saved snapshots.
- Diagnostics can distinguish edit-world validity from play-world runtime
  failures.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Run play mode directly against edit state.
  - Rejected because script and simulation mutations could corrupt saved
    project data.
- Automatically apply play-mode changes back to edit state.
  - Rejected because it makes persistence surprising and judgment-heavy.
- Make play-world objects completely unrelated to edit-world objects.
  - Rejected because diagnostics, debugging, and apply-back workflows need
    stable mapping when runtime objects derive from saved instances.

## Reopen Conditions

- Play-mode memory or startup costs make full sandboxing impractical.
- Multiplayer simulation requires a more explicit world/session model.
- Apply-back workflows require richer provenance than GUID/runtime-ID mapping.
