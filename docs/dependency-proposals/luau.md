# Luau Dependency Proposal

Status: Proposed, blocked on maintainer approval and unsafe/FFI review.

Related ADRs and docs:

- ADR 0008: Luau Scripting API
- ADR 0017: Unsafe Rust Boundary Policy
- ADR 0018: Dependency Governance
- ADR 0020: Runtime Execution Model
- ADR 0022: Runtime Frame and Event Order
- Scripting model: `docs/architecture/scripting-model.md`
- Crate map: `docs/architecture/crate-map.md`

## Decision Needed

Approve the Luau VM and tooling dependency direction before `kinetik-script-luau`
implements runtime bindings.

Luau integration is the highest-risk M4 dependency area because VM bindings may
involve FFI, vendored C/C++ code, JIT choices, sandboxing, and lifetime safety.
Installation requires explicit maintainer approval and ADR 0017 unsafe-boundary
review.

Current crate metadata was checked with `cargo info` and `cargo search` on
2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream | Proposal outcome |
| --- | ---: | --- | --- | --- | --- |
| `mlua` | `0.11.5`, with `0.12.0-rc.1` noted by Cargo | MIT | 1.80.0 | `mlua-rs/mlua` | Primary VM-binding candidate with `luau` feature, pending FFI review. |
| `full_moon` | `2.2.0` | MPL-2.0 | Unknown | `Kampfkarren/full-moon` | Defer; useful parser candidate but license and scope need review. |
| `luau-parser` | `0.2.68` | MIT | Unknown | `msix29/luau-parser` | Defer; parser-only candidate, not VM runtime. |
| `luau-analyze` | `0.0.1` | MIT | 1.85.0 | `cortesi/luau-analyze` | Reject for now; early version and MSRV above workspace. |
| `rlua` | `0.20.1` | MIT | 1.75 | `mlua-rs/rlua` | Reject for Luau runtime; Lua 5.x focused wrapper. |

## Recommendation

Evaluate `mlua` with the `luau` feature as the first VM-binding candidate, but
do not install it until a focused Luau runtime contract and unsafe/FFI review
exist.

Initial installation, if approved later, should:

- Live only in `kinetik-script-luau`.
- Prefer interpreter-only Luau before JIT/codegen features.
- Avoid raw filesystem, OS, socket, credential, and environment access.
- Expose safe Kinetik script handles and service APIs, not VM internals.
- Attribute diagnostics and logs to script asset, owning instance, frame, and
  lifecycle phase.

Parser or analyzer crates should be separate proposals. Do not combine VM
embedding, type checking, formatter/parser, and binding generation in one
installation patch.

## Ownership Boundary

- `kinetik-script` owns runtime-agnostic script lifecycle contracts.
- `kinetik-script-luau` owns Luau VM embedding, binding generation/runtime glue,
  sandbox setup, and conversion between Kinetik handles and Luau values.
- Runtime, scene, resource, signal, and diagnostics crates expose safe contracts
  consumed by scripting.

Raw VM state, stack values, userdata internals, FFI pointers, and third-party
error types must not leak into public engine APIs.

## ADR 0018 Checklist

### Why It Is Needed

ADR 0008 selects Luau as the creator-facing scripting API, and ADR 0020 includes
Luau lifecycle hooks in the runtime model. A VM bridge is eventually required,
but it must preserve sandboxing, deterministic lifecycle order, safe handles,
and editor/runtime boundaries.

### Alternatives Considered

- Implement raw Luau FFI directly in Kinetik.
  - Rejected unless a future ADR proves wrapper crates cannot satisfy the need.
- Add `mlua` immediately.
  - Deferred until lifecycle, handle, sandbox, and diagnostics contracts are
    specified.
- Enable Luau JIT/codegen immediately.
  - Deferred because safety, platform support, determinism, and debugging need
    interpreter behavior first.
- Use parser/analyzer crates as the runtime foundation.
  - Rejected because they do not embed a VM.

### License Compatibility

`mlua`, `luau-parser`, `luau-analyze`, and `rlua` report MIT. `full_moon`
reports MPL-2.0 and needs explicit license review before any use.

### Maintenance Health

`mlua` is an active Lua/Luau binding crate. Luau-specific parser/analyzer crates
vary in maturity; analyzer crates should not be part of the first VM install.

### Transitive Dependency Risk

VM bindings can bring vendored source, build scripts, and platform-specific
code. Installation PRs must include exact feature selection and
`cargo tree -e features`.

### Unsafe or FFI Exposure

This area requires explicit ADR 0017 review. Kinetik code must not introduce
unsafe or hide unsafe behind vague wrappers without approval. Any FFI boundary
must be documented and wrapped by safe Kinetik APIs.

### Platform Support

Initial target is desktop editor/runtime. JIT/codegen features should remain
disabled until platform support and sandbox implications are approved.

### Build-Time Impact

Vendored VM builds and binding generation can increase build time. Keep them
isolated to `kinetik-script-luau`.

### Runtime Size and Performance Impact

Interpreter embedding increases runtime size. The first goal is correct,
diagnosable lifecycle behavior; performance features require profiling and
approval.

### Public API Impact

Public script APIs should be Kinetik-owned Luau service and handle contracts.
No Rust VM binding types should appear in engine public APIs.

### Serialized Format Impact

No serialized-format impact is approved. Script asset references, generated
bindings, and cached analysis outputs require separate contracts.

### Crate Ownership

`kinetik-script-luau` owns Luau dependency boundaries. `kinetik-script` remains
runtime-agnostic.

## Approval Outcome

Implementation must wait for maintainer approval, feature selection, and
unsafe/FFI boundary review.
