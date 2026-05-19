# ADR 0022: Runtime Frame and Event Order

## Status

Accepted as initial project direction.

## Context

Runtime systems need predictable timing. Scripts, physics, signals, animation,
audio, rendering, diagnostics, and MCP inspection must agree on when state can
change and when events are delivered.

Without an explicit frame lifecycle, subsystems will invent incompatible timing
assumptions.

## Decision

Kinetik uses a deterministic frame lifecycle with separate variable update,
fixed simulation, deterministic signal flush points, and safe structural-change
sync points.

Initial principles:

- Runtime frame order is deterministic.
- Variable frame update and fixed timestep simulation are separate.
- Script `update(dt)` and `physics_update(fixed_dt)` are distinct lifecycle
  hooks.
- Signals/events are delivered at deterministic flush points.
- Runtime-spawned and despawned instances are applied at safe sync points, not
  mid-iteration.
- Diagnostics and logs are attributable to a frame or fixed step when possible.
- Rendering observes a coherent world snapshot.

## Initial Frame Shape

The exact micro-order may evolve, but the first runtime loop should follow this
shape:

```text
1. Poll platform and input.
2. Begin frame diagnostics/log scope.
3. Apply queued structural changes from prior safe points.
4. Run variable script update(dt).
5. Run zero or more fixed simulation steps:
   - run physics_update(fixed_dt)
   - step physics
   - collect collision/physics events
   - flush fixed-step signals/events
   - apply fixed-step safe structural changes
6. Flush frame-level signals/events.
7. Update derived transforms and world state.
8. Update animation and audio.
9. Render from a coherent world snapshot.
10. End frame cleanup.
```

## Structural Changes

Creating, deleting, reparenting, attaching scripts, and changing class-level
structure during iteration must be queued for a safe sync point unless the
operation is explicitly proven safe.

This avoids invalidating active iteration over scripts, physics bodies, signals,
or render state.

## Consequences

- Gameplay behavior is easier to reason about and test.
- Scripts cannot accidentally observe half-applied structural changes.
- Signals and physics events have predictable delivery points.
- Render, audio, and diagnostics can consume coherent snapshots.
- MCP inspection can report which frame or fixed step produced a result.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Let each subsystem choose its own timing.
  - Rejected because behavior would become hard to reason about and test.
- Apply structural changes immediately everywhere.
  - Rejected because it risks invalidating active iteration and producing
    order-dependent bugs.
- Fully specify final physics/render micro-order now.
  - Rejected because implementation should prove exact details while preserving
    the deterministic lifecycle principles.

## Reopen Conditions

- Physics integration requires a different fixed-step order.
- Rendering or animation architecture requires a different snapshot boundary.
- Multiplayer or rollback requirements introduce stricter frame-order needs.
