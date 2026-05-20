# Serialization Dependency Proposal: TOML and RON

Status: Approved for scoped installation follow-up.

Related ADRs:

- ADR 0012: Project Serialization Format
- ADR 0018: Dependency Governance
- Crate map: `docs/architecture/crate-map.md`

## Decision Needed

Approve a focused serialization dependency set for deterministic source project
files:

- `serde` with `derive` for internal contract structs.
- `toml` for `Kinetik.toml` and `*.knmanifest` parsing/writing.
- `ron` for `.knscene` and `.knprefab` parsing/writing.

This proposal is approved. Do not add these dependencies except through a
focused dependency-installation issue that records exact versions/features and
runs the required checks.

Current crate metadata was checked with `cargo info` on 2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream |
| --- | ---: | --- | --- | --- |
| `serde` | `1.0.228` | MIT OR Apache-2.0 | 1.56 | `serde-rs/serde` |
| `toml` | `1.1.2+spec-1.1.0` | MIT OR Apache-2.0 | 1.76 | `toml-rs/toml` |
| `toml_edit` | `0.25.11+spec-1.1.0` | MIT OR Apache-2.0 | 1.76 | `toml-rs/toml` |
| `ron` | `0.12.1` | MIT OR Apache-2.0 | 1.64.0 | `ron-rs/ron` |

## Recommendation

Use `serde`, `toml`, and `ron` first.

Defer `toml_edit`. Kinetik's initial source files should be deterministic
writer-owned documents, not format-preserving hand-edited documents. If editor
work later needs comment-preserving or partial TOML edits, open a separate
proposal for `toml_edit`.

## Ownership Boundary

Dependency types must not leak into public engine APIs.

- `kinetik-resource` owns TOML loading/writing for `project/assets.knmanifest`.
- `kinetik-scene` owns RON loading/writing for `.knscene` and `.knprefab`
  contract types.
- The eventual `Kinetik.toml` project settings owner must keep TOML parsing
  behind a project-settings boundary. If no crate owns that boundary cleanly
  when implementation starts, create a scoped follow-up issue before adding
  dependencies.
- Contract structs may derive `serde` internally, but public APIs should expose
  Kinetik-owned types and errors.

## ADR 0018 Checklist

### Why It Is Needed

ADR 0012 selects TOML for project settings/manifests and RON for scene/prefab
trees. Implementing deterministic load/save without mature parsers would spend
engine effort on file format tooling instead of Kinetik behavior.

### Alternatives Considered

- Manual TOML/RON parsing and writing.
  - Rejected because it increases correctness and compatibility risk.
- `toml_edit` as the first TOML dependency.
  - Deferred because format preservation is not required for generated
    deterministic manifests.
- JSON or YAML crates.
  - Rejected by ADR 0012.
- Custom scene syntax.
  - Rejected by ADR 0012.

### License Compatibility

All recommended crates report `MIT OR Apache-2.0`, matching the standard Rust
ecosystem licensing pattern and compatible with the current project direction.

### Maintenance Health

`serde` is core Rust ecosystem infrastructure. `toml` and `toml_edit` are
maintained under `toml-rs/toml`. `ron` is maintained under `ron-rs/ron`.
Before installation, rerun `cargo info` and inspect release recency if the
approval is not acted on promptly.

### Transitive Dependency Risk

Expected transitive risk is moderate and format-specific. Implementation PRs
must include `cargo tree -e features` output in PR notes for any dependency
installation. Avoid broad optional features unless required by tests.

### Unsafe or FFI Exposure

No FFI is expected. No unsafe Rust should be introduced in Kinetik code. Any
unsafe in transitive dependencies must be called out in the dependency
installation PR.

### Platform Support

The reported MSRVs are below the workspace toolchain currently used by CI.
These crates are pure Rust and suitable for desktop/editor use. Runtime platform
constraints should be reevaluated before shipping serialization into constrained
targets.

### Build-Time Impact

`serde` derive and format parser crates will increase compile time. Keep the
dependencies in document-owning crates only, and avoid exposing them through
foundational crates unless a later issue proves it is necessary.

### Runtime Size and Performance Impact

Serialization is editor/build/load-save oriented, not per-frame runtime work.
Parsing and writing should stay outside hot loops. Deterministic output should
be tested with golden fixtures once file I/O lands.

### Public API Impact

Public API impact should be limited to Kinetik-owned load/save functions,
contract structs, and error types. Do not expose `toml`, `ron`, or `serde`
types from public engine APIs.

### Serialized Format Impact

The dependency set implements the format split already accepted by ADR 0012:
TOML for project settings/manifests, RON for scene/prefab trees. Contract shape
changes still require focused issues and tests.

### Crate Ownership

Initial ownership:

- `kinetik-resource`: asset manifest TOML boundary.
- `kinetik-scene`: scene and prefab RON boundary.

Do not add a shared serialization abstraction until duplication appears in real
code. If a third crate needs the same helper logic, create a focused refactor
issue.

## Approval Outcome

Approved by maintainer direction in issue #48 on 2026-05-20.

Create a separate dependency-installation issue that adds only `serde`, `toml`,
and `ron`, records exact versions/features, and runs the full workspace checks.
`toml_edit` remains deferred unless a separate proposal approves
format-preserving TOML edits.
