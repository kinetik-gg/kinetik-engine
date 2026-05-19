# ADR 0017: Unsafe Rust Boundary Policy

## Status

Accepted as initial project direction.

## Context

Kinetik is a Rust-native game engine, so some future integrations may pressure
the codebase toward `unsafe`: graphics APIs, window handles, audio backends,
physics FFI, scripting VM bindings, SIMD, allocators, or platform-specific
systems.

Agents and humans must not use `unsafe` as a shortcut around ownership,
lifetimes, API design, or compiler errors.

The workspace already sets `unsafe_code = "forbid"`. This ADR governs how rare
exceptions may be proposed.

## Decision

`unsafe` is forbidden by default across Kinetik.

No contributor or agent may introduce `unsafe`, relax `unsafe_code = "forbid"`,
or add a dependency wrapper that hides unsafe behavior without explicit approval.

Any exception requires:

- A dedicated ADR or ADR amendment.
- A clear statement of why safe Rust cannot satisfy the requirement.
- The smallest possible unsafe boundary.
- A safe public API around the unsafe implementation.
- Documented invariants on every unsafe boundary.
- Tests that exercise the safe API and relevant failure cases.
- Review by a human maintainer.
- No unrelated changes in the same patch.

## Boundary Rules

If unsafe is ever approved:

- Unsafe code must live in a narrowly named module, not scattered through call
  sites.
- Unsafe functions must not become the normal API used by engine systems.
- The safe wrapper must validate inputs before crossing the boundary.
- The unsafe block must be as small as practical.
- The module must document ownership, aliasing, lifetime, threading, and
  initialization invariants.
- Panics, errors, and diagnostics must not leave foreign or platform state in an
  invalid condition.
- FFI handles must use typed wrappers rather than raw integers or raw pointers
  in public engine APIs.

## Agent Rules

Agents must treat unsafe like a serialized format change or dependency addition:
it requires explicit approval and architectural context.

Agents must not:

- Add `unsafe` to silence the compiler.
- Disable or weaken `unsafe_code = "forbid"`.
- Suggest broad unsafe refactors.
- Hide unsafe inside vague helpers.
- Prefer unsafe for performance before profiling proves the need.

When blocked by a Rust safety issue, agents should redesign the ownership model,
split responsibilities, introduce explicit handles, or ask for guidance.

## Consequences

- Kinetik keeps Rust safety as a default architectural property.
- Future unsafe integrations remain isolated, documented, and testable.
- The codebase avoids accumulating hidden memory-safety debt.
- Agents have clear guardrails and cannot treat unsafe like TypeScript `any`.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Allow unsafe where contributors judge it useful.
  - Rejected because unsafe debt is difficult to audit after the fact.
- Ban unsafe forever with no exception process.
  - Rejected because low-level engine integrations may eventually require a
    tightly controlled unsafe boundary.
- Allow unsafe only in dependency crates.
  - Rejected as a complete policy because Kinetik still needs to govern how
    dependency unsafe surfaces enter engine APIs.

## Reopen Conditions

- A required platform, rendering, audio, physics, or scripting integration cannot
  be implemented with the current policy.
- Performance profiling proves a safe implementation cannot meet engine
  requirements.
- Rust language or ecosystem changes provide better safety mechanisms.
