# ADR 0001: Core Stack

## Status

Accepted as initial project direction.

## Context

Kinetik is a marathon-scale game engine intended to be built by coordinated human and AI-agent contributors.

## Decision

Rust core, Luau scripting, wgpu renderer, Vello editor.

Accepted stack direction does not by itself approve dependency additions.
Dependencies must still follow the dependency governance policy.

## Consequences

- This decision shapes crate boundaries, public APIs, editor workflows, tests, and agent assignments.
- Reopening requires a follow-up ADR with alternatives and migration implications.

## Alternatives Considered

To be expanded when the decision is challenged or refined.

## Reopen Conditions

- Technical blocker discovered.
- Better option emerges with clear migration path.
- Product direction changes.
