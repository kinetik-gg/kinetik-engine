# Core Utilities Dependency Survey

Status: Survey complete, no dependency installation proposed.

Related ADRs and docs:

- ADR 0001: Core Stack
- ADR 0018: Dependency Governance
- Crate map: `docs/architecture/crate-map.md`
- Milestone roadmap: `docs/backlog/milestones.md`

## Decision Needed

Confirm that `kinetik-core` should remain dependency-free for the next
foundational milestone work.

No crate should be added by this survey. If a later implementation issue proves
that one of these utilities is necessary, open a focused dependency proposal or
installation issue for that crate and rerun current crate metadata checks.

## Current Workspace Baseline

The workspace currently has no external normal dependencies. `kinetik-core`
already owns:

- Typed non-zero ID wrappers.
- Shared error and diagnostic primitives.
- Small math/value primitives used by reflection and scenes.

That keeps foundational APIs Kinetik-owned and avoids early leakage of
third-party types into scene, reflection, serialization, editor, MCP, and
runtime contracts.

Current crate metadata was checked with `cargo info` on 2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream | Survey outcome |
| --- | ---: | --- | --- | --- | --- |
| `thiserror` | `2.0.18` | MIT OR Apache-2.0 | 1.68 | `dtolnay/thiserror` | Defer until error boilerplate becomes material. |
| `anyhow` | `1.0.102` | MIT OR Apache-2.0 | 1.68 | `dtolnay/anyhow` | Reject for engine public errors; possible future tool-only use needs separate review. |
| `uuid` | `1.20.0` observed, `1.23.1` latest noted by Cargo | Apache-2.0 OR MIT | 1.63.0 | `uuid-rs/uuid` | Defer until asset/project GUID format changes are approved. |
| `glam` | `0.32.1` | MIT OR Apache-2.0 | 1.68.2 | `bitshifter/glam-rs` | Defer to renderer/runtime math boundary, not `kinetik-core`. |
| `bitflags` | `2.11.1` | MIT OR Apache-2.0 | 1.56.0 | `bitflags/bitflags` | Defer until a real flag set appears. |
| `indexmap` | `2.11.4` observed, `2.14.0` latest noted by Cargo | Apache-2.0 OR MIT | 1.63 | `indexmap-rs/indexmap` | Defer; current deterministic ordering uses owned vectors and `BTreeMap`/`BTreeSet`. |
| `smallvec` | `1.15.1` stable, `2.0.0-alpha.12` latest noted by Cargo | MIT OR Apache-2.0 | Unknown for 1.15.1 | `servo/rust-smallvec` | Defer until profiling shows allocation pressure. |
| `slotmap` | `1.1.1` | Zlib | 1.58.0 | `orlp/slotmap` | Reject for now; typed ID storage policy is already Kinetik-owned. |

## Recommendation

Keep `kinetik-core` dependency-free.

Approve no new core utility dependencies at this time. This is the smallest
safe decision because the current code already covers the foundational needs
without third-party public API exposure, unsafe review, serialized-format
impact, or build-time cost.

If future work creates concrete pressure:

- Prefer a crate-local boundary over adding broad utilities to `kinetik-core`.
- Keep third-party types out of public Kinetik APIs unless a separate ADR
  approves the leak.
- Require a focused proposal for the exact crate, feature set, owner crate, and
  tests before installation.

## Ownership Boundary

`kinetik-core` should continue to own foundation primitives directly.

Candidate future boundaries:

- Error derives, if approved, belong only where hand-written error impls become
  maintenance noise. Public error enums remain Kinetik-owned.
- Math acceleration belongs behind renderer/runtime extraction boundaries unless
  a future ADR approves replacing core math primitives.
- GUID parsing/generation belongs behind asset/project identity boundaries and
  must not silently change serialized identity formats.
- Storage/indexing helpers belong to the crate with the measured need, not a
  shared utility layer by default.

## ADR 0018 Checklist

### Why It Is Needed

M4 requires dependency surveys before dependency-backed implementation. This
survey exists to prevent agents from casually adding foundational helper crates
while later milestones are still defining project, diagnostics, command,
runtime, editor, and MCP contracts.

No immediate dependency need was found.

### Alternatives Considered

- Add `thiserror` now for all error enums.
  - Deferred because the current hand-written errors are small, explicit, and
    dependency-free.
- Add `anyhow` for flexible errors.
  - Rejected for engine public APIs because Kinetik needs structured errors and
    diagnostics, not opaque error aggregation.
- Add `uuid` for GUIDs.
  - Deferred because stable project identity and serialized GUID formats are
    architecture-sensitive and already represented by Kinetik-owned non-zero
    wrappers.
- Add `glam` for math.
  - Deferred because public math/value primitives already exist and renderer or
    runtime internals can evaluate math dependencies at their own boundaries.
- Add `bitflags`, `indexmap`, `smallvec`, or `slotmap` as general utilities.
  - Deferred or rejected because no current issue has demonstrated a concrete
    need that outweighs dependency surface area.

### License Compatibility

Most surveyed crates use `MIT OR Apache-2.0`, which is compatible with the
workspace license direction. `slotmap` reports `Zlib`; that may still be
permissive, but it should receive explicit human review before any future use.

### Maintenance Health

The surveyed crates are broadly used Rust ecosystem crates. That is not enough
to justify installation. Before any future install issue, rerun `cargo info`,
review release recency, and include the exact crate version and feature set in
the PR notes.

### Transitive Dependency Risk

The current workspace has no external normal dependencies. Installing any core
utility would create the first external normal dependency path for foundational
crates.

Many candidates have optional features that add transitive dependencies. Future
install issues should disable optional features unless required, and include
`cargo tree -e features` output in PR notes.

### Unsafe or FFI Exposure

No unsafe Rust should be introduced in Kinetik code by core utility adoption.
Future proposals must call out any unsafe code in transitive dependencies.

No FFI exposure is expected from the surveyed crates.

### Platform Support

The surveyed crates report MSRVs below the workspace `rust-version` of 1.80, or
no MSRV for the checked stable `smallvec` release. Future install issues should
rerun metadata checks and verify target support for desktop editor, runtime, and
eventual constrained runtime targets.

### Build-Time Impact

Keeping `kinetik-core` dependency-free avoids adding compile time to every crate
that depends on it. Derive macros such as `thiserror` are especially likely to
affect incremental builds across common foundational crates.

### Runtime Size and Performance Impact

No current runtime performance issue justifies adding a core utility. Data
structure and math dependencies should be justified by profiling, deterministic
behavior needs, or a concrete subsystem boundary.

### Public API Impact

No public API impact is proposed.

Future dependency-backed implementation must keep external types out of public
Kinetik APIs unless an ADR or approved proposal explicitly permits them.

### Serialized Format Impact

No serialized-format impact is proposed.

Future use of `uuid` or any math/storage crate that changes project, scene,
manifest, prefab, or diagnostic serialization must go through the serialized
format gate before implementation.

### Crate Ownership

No crate owns a new dependency boundary from this survey.

Future ownership should be specific:

- `kinetik-core`: only foundational utilities that every dependent crate truly
  needs and that do not leak third-party types.
- Domain crates: subsystem-specific helpers that can remain internal.
- Tooling crates: flexible error/reporting helpers only when kept out of engine
  runtime and public contract APIs.

## Approval Outcome

This survey recommends no dependency approval and no installation issue.

If maintainers disagree, create a separate issue for the chosen crate with the
exact intended owner, features, public API boundary, and required checks.
