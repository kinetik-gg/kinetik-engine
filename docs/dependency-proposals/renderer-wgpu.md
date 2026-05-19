# Renderer Dependency Proposal: wgpu Stack

Status: Approved direction, blocked on MSRV/version installation decision.

Related ADRs and docs:

- ADR 0001: Core Stack
- ADR 0006: Renderer and Shader Graph
- ADR 0018: Dependency Governance
- Crate map: `docs/architecture/crate-map.md`
- Rendering model: `docs/architecture/rendering-model.md`

## Decision Needed

Approve the renderer dependency direction for `kinetik-render` before any GPU
backend implementation starts.

Two decisions are required before installation:

1. Whether to raise the workspace `rust-version` from 1.80 to the current
   `wgpu` stack MSRV.
2. Whether the first renderer implementation should use the latest `wgpu` stack
   or deliberately pin an older compatible line for the initial primitive slice.

The `wgpu` renderer direction is approved. Do not add dependencies until the
MSRV/version decision is explicit in a focused installation issue.

Current crate metadata was checked with `cargo info` on 2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream | Proposal outcome |
| --- | ---: | --- | --- | --- | --- |
| `wgpu` | `29.0.3` | MIT OR Apache-2.0 | 1.87.0 | `gfx-rs/wgpu` | Recommended renderer backend after MSRV decision. |
| `wgpu` older line | `24.0.5` checked | MIT OR Apache-2.0 | Unknown | `gfx-rs/wgpu` | Possible temporary pin if Rust 1.80 must remain fixed. |
| `naga` | `29.0.3` | MIT OR Apache-2.0 | 1.87 | `gfx-rs/wgpu` | Defer direct dependency unless shader graph work needs explicit shader IR access. |
| `bytemuck` | `1.25.0` | Zlib OR Apache-2.0 OR MIT | Unknown | `Lokathor/bytemuck` | Defer until GPU buffer layout code needs POD casts. |
| `encase` | `0.12.0` | MIT-0 | 1.77 | `teoxoy/encase` | Defer until uniform/storage buffer layout complexity justifies it. |
| `raw-window-handle` | `0.6.2` | MIT OR Apache-2.0 OR Zlib | 1.64 | `rust-windowing/raw-window-handle` | Usually arrives through window/editor integration; do not expose from authoring APIs. |
| `pollster` | `0.4.0` | Apache-2.0/MIT | Unknown | `zesterer/pollster` | Defer; use only for tiny examples/tests if async renderer init lacks a better harness. |
| `glam` | `0.32.1` | MIT OR Apache-2.0 | 1.68.2 | `bitshifter/glam-rs` | Defer from public renderer contracts; consider internal extraction math only in a later proposal. |

## Recommendation

Use `wgpu` as the renderer backend owned by `kinetik-render`, but do not install
it until maintainers choose an MSRV path.

Recommended installation shape after approval:

- Add `wgpu` only to `kinetik-render` at first.
- Keep `wgpu`, `naga`, surface, adapter, device, queue, pipeline, texture, and
  shader-module types out of public authoring APIs.
- Start with WGSL shader modules and a simple forward subset aligned with ADR
  0006.
- Defer direct `naga`, `bytemuck`, `encase`, `glam`, and `pollster`
  dependencies until a focused implementation issue proves the need.
- Keep window/surface ownership coordinated with the future editor/window/UI
  dependency proposal instead of making `kinetik-render` own editor windows.

If the project keeps Rust 1.80 for now, open a follow-up installation issue that
evaluates a deliberate older `wgpu` pin and records why that pin is acceptable.
If the project accepts Rust 1.87 or newer, open a follow-up installation issue
for the latest compatible `wgpu` line and record exact features.

## Ownership Boundary

`kinetik-render` owns the GPU backend boundary.

Renderer-owned internals may include:

- Backend instance/device/queue/surface lifecycle.
- GPU resource caches.
- Render pipeline and bind group layout construction.
- WGSL shader modules.
- Render extraction output consumed by GPU code.
- Render diagnostics for missing or invalid cameras, meshes, materials,
  textures, shaders, lights, and backend state.

Renderer-owned public contracts should remain Kinetik types:

- Renderable scene extraction inputs.
- Camera, light, mesh, material, and texture references.
- Render settings and diagnostics.
- Stable resource identifiers and human-readable paths.

Public authoring APIs must not expose raw `wgpu` or `naga` types unless a later
ADR explicitly approves that leak.

## ADR 0018 Checklist

### Why It Is Needed

ADR 0006 selects a modern `wgpu` 3D renderer with practical PBR, HDR direction,
render graph organization, shader graph to WGSL output, and structured render
diagnostics.

Implementing GPU rendering without a mature backend would spend engine effort
on graphics API abstraction instead of Kinetik renderer behavior. `wgpu` is
already accepted as project direction, but ADR 0018 requires explicit approval
before dependency installation.

### Alternatives Considered

- Implement a custom graphics abstraction directly over platform APIs.
  - Rejected because it would be too large for the first playable template
    path and would duplicate `wgpu`'s cross-platform backend work.
- Use a toy software or immediate-mode renderer for early templates.
  - Rejected because ADR 0006 says early visible-scene work must not block the
    future PBR/render-graph/material-graph direction.
- Add `naga` directly now.
  - Deferred because `wgpu` already covers WGSL validation needs for the first
    visible slice. Direct shader IR access should wait for shader graph work.
- Add `bytemuck` or `encase` immediately for buffer layout.
  - Deferred because first renderer code should prove the concrete data layout
    before adding helper crates.
- Add `glam` to replace Kinetik math types.
  - Deferred because public math/value contracts are already Kinetik-owned and
    replacement would affect public APIs and serialized/property contracts.

### License Compatibility

`wgpu`, `naga`, and `glam` report `MIT OR Apache-2.0`. `bytemuck` and
`raw-window-handle` include Zlib alternatives, and `encase` reports `MIT-0`.

These licenses appear permissive, but the installation PR must record the exact
licensed dependency set and any transitive licenses from `cargo tree`.

### Maintenance Health

`wgpu` and `naga` are maintained under the `gfx-rs/wgpu` project and are active
Rust graphics ecosystem foundations. Their rapid release cadence also means
Kinetik must treat MSRV and API churn as part of the dependency decision.

Before installation, rerun `cargo info` for the chosen exact versions.

### Transitive Dependency Risk

`wgpu` brings a substantial transitive dependency tree for backend support,
shader handling, platform integration, and optional web support.

Implementation PR notes must include:

- Exact `wgpu` version.
- Enabled features.
- `cargo tree -e features` output.
- Any platform-specific features intentionally disabled.

Avoid broad optional features unless required by the focused renderer issue.

### Unsafe or FFI Exposure

No unsafe Rust should be introduced in Kinetik code by dependency installation.

Graphics backend crates may contain unsafe or platform FFI internally. The
installation PR must call out any unsafe/FFI exposure visible in the dependency
tree and keep it behind the `kinetik-render` boundary.

### Platform Support

The current latest `wgpu` reports MSRV 1.87.0, while the workspace currently
declares Rust 1.80. That is a blocker for installing the latest `wgpu` line.

The renderer target remains desktop-first for the first 3D templates. Web,
mobile, and constrained runtime targets should not drive the first installation
feature set unless maintainers explicitly widen the platform scope.

### Build-Time Impact

`wgpu` will materially increase compile time and dependency download size. Keep
it out of foundational crates and avoid adding renderer dependencies to editor,
app, scene, resource, or core crates unless a separate boundary issue justifies
it.

### Runtime Size and Performance Impact

GPU backend initialization, shader compilation, resource creation, and pipeline
creation should stay outside hot per-frame paths where possible. The first
renderer implementation should test deterministic extraction and diagnostics
before optimizing pipeline caching.

### Public API Impact

The intended public API impact is Kinetik-owned renderer contracts only.

Do not expose:

- `wgpu::Device`, `Queue`, `Texture`, `Buffer`, `Surface`, `Adapter`, or
  pipeline types.
- `naga` IR types.
- Raw window handles in authoring APIs.

Window and surface integration belongs at the renderer/editor/app boundary and
should be coordinated with the editor/window/UI dependency proposal.

### Serialized Format Impact

No serialized-format impact is approved by this proposal.

Material, mesh, texture, shader graph, and render settings serialization remain
separate architecture-sensitive issues. Any file format or reflected property
changes must go through their own ADR or serialized-format gate.

### Crate Ownership

Initial ownership:

- `kinetik-render`: owns `wgpu` and direct renderer helper dependencies.
- `kinetik-editor`: may later own window/editor UI dependencies and pass
  renderer-compatible surface inputs through an approved boundary.
- `kinetik-resource`: owns asset identity and source asset references, not GPU
  resource objects.
- `kinetik-scene`: owns instance data consumed by render extraction, not GPU
  state.

Do not add a shared rendering utility crate until duplication appears in real
code and a focused refactor issue justifies the split.

## Approval Outcome

Approved by maintainer direction in issue #48 on 2026-05-20.

Create a separate dependency-installation issue that:

- Resolves the Rust 1.80 versus `wgpu` 29 MSRV mismatch.
- Adds only the approved renderer dependencies.
- Records exact versions and features.
- Includes `cargo tree -e features` in PR notes.
- Runs the full required checks for the implementation level.
