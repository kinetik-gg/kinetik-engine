# ADR 0002: Instance Scene Model

## Status

Accepted as initial project direction.

## Context

Kinetik is a marathon-scale game engine intended to be built by coordinated human and AI-agent contributors.

## Decision

Instances are the authoring model. A scene owns a serialized instance hierarchy
with one root `Game` instance.

Kinetik Studio scaffolds default top-level service instances under `Game`:

```text
Game
  Workspace
  Prefabs
  Scripts
  UI
  Lighting
  Audio
  Physics
  Assets
  Packages
```

These default services are real serialized instances, not hidden editor-only
singletons. The Explorer panel must list them as normal workable instances so
creators can discover engine capabilities by inspecting the default hierarchy.

Users should not need to manually create these services for a new project. The
editor creates them during project or scene scaffolding and keeps them visible,
addressable, scriptable where appropriate, and available to MCP tools.

## Consequences

- Scene inspection is straightforward for humans, agents, and Git review.
- MCP tools can address default services through stable scene paths such as
  `/Game/Lighting` and `/Game/Workspace`.
- Editor features should prefer visible service instances over hidden global
  settings panels when the concept belongs to the scene.
- This decision shapes crate boundaries, public APIs, editor workflows, tests,
  and agent assignments.
- Reopening requires a follow-up ADR with alternatives and migration implications.

## Alternatives Considered

- Hidden editor/runtime services.
  - Rejected as the default because they make scene capabilities harder to
    discover, inspect, automate, and review.
- Implicit services created only at runtime.
  - Rejected because they obscure project state and weaken editor/MCP
    introspection.

## Reopen Conditions

- Technical blocker discovered.
- Better option emerges with clear migration path.
- Product direction changes.
