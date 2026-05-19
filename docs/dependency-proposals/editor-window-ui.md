# Editor, Window, and UI Dependency Proposal

Status: Proposed, blocked on maintainer approval and MSRV direction.

Related ADRs and docs:

- ADR 0001: Core Stack
- ADR 0011: Agentic Editor MCP Contract
- ADR 0014: Editor Command and Semantic Change Model
- ADR 0018: Dependency Governance
- ADR 0019: Edit, Play, and Runtime State Boundaries
- Crate map: `docs/architecture/crate-map.md`

## Decision Needed

Approve the first editor/window/UI dependency direction before Kinetik Studio
shell work begins.

The accepted stack names Vello editor direction and `winit` is already listed
as preapproved direction in ADR 0018, but that is not installation approval.
Latest Vello also creates an MSRV decision because `vello` 0.9 reports Rust
1.88 while the workspace declares Rust 1.80.

Current crate metadata was checked with `cargo info` on 2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream | Proposal outcome |
| --- | ---: | --- | --- | --- | --- |
| `winit` | `0.30.13`, with `0.31.0-beta.2` noted by Cargo | Apache-2.0 | 1.70.0 | `rust-windowing/winit` | Recommended first window/event-loop candidate after approval. |
| `vello` | `0.9.0` | Apache-2.0 OR MIT | 1.88 | `linebender/vello` | Recommended direction, blocked by MSRV decision. |
| `vello` older line | `0.3.0` checked | Apache-2.0 OR MIT | 1.75 | `linebender/vello` | Possible temporary pin if Rust 1.80 remains fixed. |
| `accesskit` | `0.23.0`, with `0.24.0` noted by Cargo | MIT OR Apache-2.0 | 1.77.2 | `AccessKit/accesskit` | Defer until editor UI accessibility contract exists. |
| `raw-window-handle` | `0.6.2` | MIT OR Apache-2.0 OR Zlib | 1.64 | `rust-windowing/raw-window-handle` | Prefer transitive via `winit`/renderer boundary unless direct use is required. |

## Recommendation

Use `winit` for the editor shell event loop and window creation once approved.
Use Vello for editor UI rendering only after maintainers choose an MSRV path or
approve a deliberate older Vello pin.

Do not install `accesskit` directly in the first shell issue unless that issue
also defines an accessibility data contract. Do not let editor UI dependencies
become runtime-domain dependencies.

## Ownership Boundary

- `kinetik-editor` owns Kinetik Studio windows, panels, editor input routing,
  editor UI rendering, selection, active document session, and MCP server host.
- `kinetik-render` owns runtime/viewport GPU rendering and should be reused by
  the editor viewport where practical.
- `kinetik-ui` owns runtime UI model contracts, not editor widgets.
- Runtime crates must not depend on editor/window/UI crates.

Third-party window, event, UI, accessibility, and raw handle types should not
leak into engine authoring APIs.

## ADR 0018 Checklist

### Why It Is Needed

M16 and later editor milestones require an actual window shell and panel UI, but
ADR 0011 and ADR 0014 require editor actions to go through project/session and
command surfaces rather than private UI-only mutation paths.

### Alternatives Considered

- Build a custom platform window layer.
  - Rejected because `winit` is accepted direction and avoids platform work
    before editor behavior is proven.
- Use Vello latest immediately.
  - Deferred because the latest checked line reports Rust 1.88.
- Use a retained-mode UI crate as the core editor model.
  - Deferred because Kinetik needs command/session contracts first; UI widgets
    must consume engine/editor state, not own it.
- Add accessibility infrastructure immediately.
  - Deferred until editor UI tree ownership is specified.

### License Compatibility

Surveyed licenses are permissive. `raw-window-handle` includes Zlib as an
alternative; installation PRs must record exact transitive licenses.

### Maintenance Health

`winit` is the standard Rust windowing crate. Vello is active Linebender
infrastructure but version/MSRV churn must be treated as an explicit decision.

### Transitive Dependency Risk

Windowing and UI crates bring platform-specific dependencies. Installation PRs
must include `cargo tree -e features` and record selected platform features.

### Unsafe or FFI Exposure

Windowing and GPU UI rendering may involve dependency-internal platform FFI.
Kinetik code must remain unsafe-free unless ADR 0017 exception rules are
followed.

### Platform Support

Initial editor target is desktop. Web/mobile editor support is out of scope for
the first shell unless maintainers explicitly widen scope.

### Build-Time Impact

Vello and its GPU stack will materially increase build time. Keep these
dependencies isolated to editor/render boundaries.

### Runtime Size and Performance Impact

Editor UI dependencies should not increase standalone runtime size unless a
later issue proves shared runtime UI needs.

### Public API Impact

No public API impact is approved. Editor APIs should expose Kinetik-owned
session, command, panel, selection, and diagnostic concepts.

### Serialized Format Impact

No serialized-format impact is approved. Layout persistence and editor settings
require separate contracts.

### Crate Ownership

Initial ownership should be `kinetik-editor` for `winit`/Vello editor UI, with
surface interoperability coordinated with `kinetik-render`.

## Approval Outcome

Implementation must wait for maintainer approval and an MSRV decision for
Vello. Create a separate installation issue before adding any crates.
