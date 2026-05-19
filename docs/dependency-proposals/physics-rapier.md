# Physics Dependency Proposal: Rapier

Status: Approved direction, blocked on MSRV/version installation decision.

Related ADRs and docs:

- ADR 0007: Physics Authoring
- ADR 0017: Unsafe Rust Boundary Policy
- ADR 0018: Dependency Governance
- ADR 0022: Runtime Frame and Event Order
- Physics model: `docs/architecture/physics-model.md`
- Crate map: `docs/architecture/crate-map.md`

## Decision Needed

Approve Rapier-backed 3D physics dependencies for `kinetik-physics` before
physics implementation starts.

The latest checked `rapier3d` line reports Rust 1.86 while the workspace
declares Rust 1.80, so installation requires either an MSRV change or a
deliberate older-version pin.

Current crate metadata was checked with `cargo info` on 2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream | Proposal outcome |
| --- | ---: | --- | --- | --- | --- |
| `rapier3d` | `0.32.0` | Apache-2.0 | 1.86 | `dimforge/rapier` | Recommended backend after MSRV decision. |
| `parry3d` | `0.26.1` | Apache-2.0 | Unknown | `dimforge/parry` | Prefer transitive via Rapier unless collision-only code needs direct use. |
| `nalgebra` | `0.34.2` | Apache-2.0 | 1.87.0 | `dimforge/nalgebra` | Keep behind physics boundary; do not expose in public Kinetik APIs. |

## Recommendation

Use `rapier3d` as the physics backend owned by `kinetik-physics` once approved.
Do not install `parry3d` or `nalgebra` directly unless a focused issue proves a
need outside Rapier's public surface.

Start with deterministic, single-threaded physics integration. Defer Rapier's
`parallel`, SIMD, serialization, debug-render, and profiler features until
dedicated issues require them.

## Ownership Boundary

- `kinetik-physics` owns Rapier worlds, colliders, rigid bodies, query
  pipelines, event extraction, and conversion between Kinetik and Rapier types.
- `kinetik-scene` owns authored instance data and reflected properties.
- `kinetik-core` owns public math/value primitives.
- `kinetik-render` may later consume debug visualization data, not Rapier
  internals.

Public authoring APIs must expose Kinetik-owned body, collider, layer, material,
event, and diagnostic types rather than Rapier, Parry, or Nalgebra types.

## ADR 0018 Checklist

### Why It Is Needed

ADR 0007 selects instance-based physics backed by Rapier. ADR 0022 requires
physics to participate in deterministic fixed-step lifecycle and event flush
points.

### Alternatives Considered

- Custom physics implementation.
  - Rejected because it would dominate early engine work.
- Collision-only Parry first.
  - Deferred because ADR 0007 selects full instance-authored physics backed by
    Rapier.
- Expose Rapier objects directly.
  - Rejected because public APIs should remain Kinetik-owned and editor/MCP
    diagnostics need stable instance references.

### License Compatibility

Rapier, Parry, and Nalgebra report Apache-2.0. Installation PRs must record the
full transitive license set.

### Maintenance Health

Rapier and its Dimforge ecosystem are active Rust physics foundations. Their
MSRV now exceeds the workspace setting, so toolchain policy must be resolved.

### Transitive Dependency Risk

Rapier brings a substantial math/collision dependency tree. Installation PRs
must include `cargo tree -e features` and avoid optional features unless needed.

### Unsafe or FFI Exposure

Kinetik code must remain unsafe-free. Any dependency-internal unsafe must remain
behind `kinetik-physics` and be called out in PR notes.

### Platform Support

Initial support is desktop runtime/editor. Determinism-sensitive features should
be chosen deliberately; `enhanced-determinism` needs a focused follow-up test
plan if enabled.

### Build-Time Impact

Physics dependencies will increase build time. Keep them out of non-physics
crates unless a separate boundary issue justifies it.

### Runtime Size and Performance Impact

Start single-threaded and deterministic. Parallelism, SIMD, and profiling
features require explicit approval and tests.

### Public API Impact

Public APIs should expose Kinetik-owned physics contracts. No Rapier, Parry, or
Nalgebra types should leak into authoring APIs without an ADR.

### Serialized Format Impact

No serialized-format impact is approved. Physics instance properties and
material serialization require separate contracts and tests.

### Crate Ownership

`kinetik-physics` owns the dependency boundary. Scene, runtime, editor, and MCP
surfaces consume Kinetik physics contracts.

## Approval Outcome

Approved by maintainer direction in issue #48 on 2026-05-20.

Create a separate installation issue before adding any crates. That issue must
resolve the Rapier MSRV/version decision, record exact features, and keep
Rapier, Parry, and Nalgebra types behind the `kinetik-physics` boundary.
