# Runtime/App Dependency Survey

Status: Approved survey, no dependency installation approved.

Related ADRs and docs:

- ADR 0018: Dependency Governance
- ADR 0019: Edit, Play, and Runtime State Boundaries
- ADR 0020: Runtime Execution Model
- ADR 0022: Runtime Frame and Event Order
- Crate map: `docs/architecture/crate-map.md`

## Decision Needed

Confirm that the first `kinetik-app` and runtime frame work should remain
dependency-light until M5 runtime/frame contracts define the required surfaces.

No runtime/app dependency should be installed from this survey. The initial
runtime direction is deterministic and single-threaded, so broad async,
parallelism, scheduling, and synchronization crates are premature.

Current crate metadata was checked with `cargo info` on 2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream | Survey outcome |
| --- | ---: | --- | --- | --- | --- |
| `tracing` | `0.1.44` | MIT | 1.65.0 | `tokio-rs/tracing` | Defer until diagnostics/log attribution contract lands. |
| `tracing-subscriber` | `0.3.23` | MIT | 1.65.0 | `tokio-rs/tracing` | Defer to app/editor executable setup, not core runtime contracts. |
| `crossbeam-channel` | `0.5.15` | MIT OR Apache-2.0 | 1.60 | `crossbeam-rs/crossbeam` | Defer; first runtime is single-threaded. |
| `parking_lot` | `0.12.5` | MIT OR Apache-2.0 | 1.71 | `Amanieu/parking_lot` | Defer; avoid lock dependencies until threaded ownership is approved. |
| `pollster` | `0.4.0` | Apache-2.0/MIT | Unknown | `zesterer/pollster` | Defer to renderer/window smoke tests if needed. |

## Recommendation

Do not approve new runtime/app dependencies yet.

The next implementation work should first define `RuntimeWorld`,
`FrameScheduler`, diagnostics/log attribution, and app-loop ownership in M5.
After that, focused installation issues can decide whether `tracing` belongs in
runtime/app, editor executables, tests, or not at all.

## Ownership Boundary

- `kinetik-app` owns standalone runtime orchestration and subsystem lifecycle.
- Runtime-domain crates expose deterministic state and diagnostics, not editor
  process state.
- `kinetik-editor` may host an editor app loop, but runtime crates must not
  depend on editor or MCP implementation.

Third-party logging, scheduling, channel, or synchronization types must not leak
into public runtime contracts without explicit approval.

## ADR 0018 Checklist

### Why It Is Needed

M4 requires dependency review before runtime/app crates add external libraries.
Runtime timing affects scripts, physics, signals, diagnostics, rendering, and
future MCP inspection.

### Alternatives Considered

- Add `tracing` immediately.
  - Deferred because ADR 0015 diagnostics and ADR 0022 frame attribution need a
    concrete contract before log spans become architectural.
- Add channels or locks now.
  - Deferred because ADR 0020 selects a deterministic single-threaded core for
    the initial runtime.
- Add an async executor.
  - Deferred because the first frame scheduler should not inherit async runtime
    behavior before lifecycle order is specified.

### License Compatibility

Surveyed crates use permissive MIT or Apache-compatible licenses. Future
installation PRs must record exact transitive licenses.

### Maintenance Health

The surveyed crates are common Rust ecosystem crates. Their maturity does not
justify installation without a concrete runtime contract.

### Transitive Dependency Risk

`tracing-subscriber` and async/synchronization support can add nontrivial
transitive dependencies. Future install issues must include
`cargo tree -e features` output and avoid optional features unless required.

### Unsafe or FFI Exposure

No unsafe Rust should be introduced in Kinetik code. Future proposals must call
out dependency-internal unsafe where relevant.

### Platform Support

The surveyed crates appear compatible with the current Rust 1.80 workspace
where MSRVs are reported. Platform support still depends on the eventual app
targets and should be checked at installation time.

### Build-Time Impact

No build-time increase is justified now. Keep app-loop dependencies out of
foundational crates.

### Runtime Size and Performance Impact

The first runtime should prove deterministic frame order before optimizing
logging, synchronization, or scheduling infrastructure.

### Public API Impact

No public API impact is proposed. Runtime/app public APIs should remain
Kinetik-owned.

### Serialized Format Impact

No serialized-format impact is proposed.

### Crate Ownership

No crate owns a new dependency from this survey. Later approvals should assign
ownership to `kinetik-app`, a runtime-domain crate, or executable-only tooling
explicitly.

## Approval Outcome

Approved by maintainer direction in issue #48 on 2026-05-20.

This survey approves the dependency-light runtime/app direction. No runtime/app
dependency installation is approved yet; revisit after M5 runtime/frame
contracts identify a concrete dependency-backed need.
