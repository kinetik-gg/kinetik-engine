# DiagnosticsStore Contract

## Purpose

Define the shared current-health diagnostics store used by project validation,
commands, resources, runtime, editor panels, MCP, tests, builds, and repair
workflows.

## Owning Crates

- `kinetik-core`: diagnostic primitives, stable codes, severity, source,
  location, blocking scope, and repairability flags.
- Future project/runtime owner crates: stores scoped to project health and
  runtime health.
- `kinetik-editor`: presentation, filtering UI state, and command-driven repair
  entry points.
- MCP implementation: read-only diagnostics commands and command-backed repair
  requests.

## Store Contract

`DiagnosticsStore` contains current health records, not chronological logs.

Required capabilities:

- Insert, replace, and clear diagnostics by stable owner scope.
- Query by severity, source, blocking scope, asset, instance, scene path,
  script path, property path, and repairability.
- Provide stable diagnostic codes for tests and automation.
- Distinguish edit/project diagnostics from runtime/play diagnostics.
- Return deterministic ordering for UI, MCP, and tests.

Logs may reference diagnostics, but logs are not the diagnostics store.

## Dependency Boundaries

No new dependency is approved by this contract. Runtime/app logging dependencies
remain deferred by `docs/dependency-proposals/runtime-app.md`.

Diagnostics APIs expose Kinetik-owned records, not `tracing` spans, log records,
UI table models, or MCP transport types.

## Serialized-Format Impact

No serialized-format impact is approved.

Golden diagnostic fixtures may be added by later issues for stable command,
serialization, manifest, and import diagnostics.

## Diagnostics Behavior

Every diagnostic should include as many of these fields as are known:

- Stable code.
- Severity.
- Source system.
- Human-readable message.
- Blocking scope.
- Instance GUID and scene path.
- Asset GUID and `res://` path.
- Script path and source range.
- Property path.
- Suggested fix when safe.
- Whether automated repair is allowed.

Repair commands must go through command/change-record contracts.

## Public API Constraints

- Public diagnostic records must remain stable enough for tests and MCP clients.
- UI-specific filtering, sorting, and expansion state stays in editor code.
- MCP may expose diagnostics, but it must not own diagnostic storage.

## Follow-Up Issues

- M6 project diagnostics store.
- M10 command validation diagnostics.
- M11 resource reference diagnostics.
- M14 diagnostics listing through MCP.
