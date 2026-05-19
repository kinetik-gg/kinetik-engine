# ADR 0018: Dependency Governance

## Status

Accepted as initial project direction.

## Context

Kinetik is a long-lived engine. Dependencies affect architecture, build times,
binary size, licensing, security, platform support, unsafe exposure, serialized
formats, public APIs, and contributor comprehension.

Agents must not add dependencies as casual conveniences.

## Decision

Kinetik dependencies require explicit approval, documented rationale,
license/safety review, and boundary ownership.

External types should not leak into public engine APIs unless deliberately
approved. Kinetik should wrap dependencies at subsystem boundaries.

Examples:

- `kinetik-render` may use `wgpu` internally, but normal scene authoring should
  not expose `wgpu` types.
- `kinetik-physics` may use Rapier internally, but instance-authored physics
  should expose Kinetik types.
- `kinetik-audio` may use an audio backend internally, but gameplay code should
  use Kinetik audio abstractions.
- `kinetik-script-luau` owns Luau VM integration so raw VM details do not leak
  through runtime APIs.
- RON/TOML parser dependencies belong behind serialization and manifest loading
  boundaries.

## Dependency Review Checklist

Every new dependency proposal must state:

- Why it is needed.
- Alternatives considered.
- License compatibility.
- Maintenance health.
- Transitive dependency risk.
- Unsafe or FFI exposure.
- Platform support.
- Build-time impact.
- Runtime size and performance impact.
- Whether it affects public APIs.
- Whether it affects serialized formats.
- Which crate owns the dependency boundary.

## Preapproved Direction Is Not Installation Approval

Some technologies are accepted as project direction:

- Rust.
- Luau.
- `wgpu`.
- Vello.
- Rapier.
- `winit`.
- Kira preferred for audio.
- TOML for project settings and manifests.
- RON for scenes and prefabs.

These decisions do not authorize adding crates casually. Implementation patches
must still add dependencies deliberately, in the crate that owns the boundary,
with focused tests and no unrelated changes.

## Agent Rules

Agents must not:

- Add dependencies without explicit approval.
- Add a dependency to avoid writing a small focused implementation.
- Add broad helper crates that obscure simple logic.
- Expose third-party types from public Kinetik APIs without approval.
- Add a dependency that changes serialized formats, runtime behavior, or public
  API shape without an ADR or ADR amendment.

When a dependency seems useful, agents should present the tradeoff and wait for
approval.

## Consequences

- Kinetik keeps ownership of its public architecture.
- Dependency additions remain reviewable and purposeful.
- Third-party churn is contained behind crate boundaries.
- Build performance, binary size, licensing, and safety risks stay visible.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Allow dependencies whenever they are popular or convenient.
  - Rejected because engine foundations become difficult to audit and maintain.
- Ban all dependencies until late development.
  - Rejected because rendering, physics, scripting, audio, windowing, and
    serialization require mature ecosystem support.
- Allow dependencies only in leaf crates.
  - Rejected as too rigid; some foundational crates may need carefully approved
    dependencies.

## Reopen Conditions

- Dependency review overhead blocks necessary implementation progress.
- A workspace dependency policy tool or automation changes the governance model.
- Platform support requirements require a different boundary strategy.
