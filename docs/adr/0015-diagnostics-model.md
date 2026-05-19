# ADR 0015: Diagnostics Model

## Status

Accepted as initial project direction.

## Context

Kinetik needs errors that are useful to creators, tests, editor tools, builds,
and agents. Plain logs are not enough for project health because logs describe
what happened over time, while diagnostics describe what is currently wrong.

Commands, importers, reflection validation, scripts, scenes, prefabs, bundles,
and MCP tools all need a shared way to report failures and repairable problems.

## Decision

Kinetik uses structured diagnostics with stable codes and object/property/asset
references as the shared error reporting model for editor, runtime, importers,
commands, MCP, tests, and builds.

Diagnostics are current project or runtime health records. Logs are chronological
events. The two may reference each other, but they are not the same system.

## Diagnostic Shape

A diagnostic should include:

- Stable code.
- Severity.
- Human-readable message.
- Source system.
- Blocking scope.
- Related instance GUID and scene path when applicable.
- Related asset GUID and project path when applicable.
- Related script path and source range when applicable.
- Related property path when applicable.
- Suggested fix when safe.
- Whether an agent may attempt repair.

Example:

```text
code: KT_ASSET_MISSING_SOURCE
severity: Error
source: AssetImporter
blocking: Import
asset: res://assets/models/tree.glb
message: Source asset is missing.
suggested_fix: Restore the file or remove the manifest entry.
agent_repair: allowed
```

Example:

```text
code: KT_PROPERTY_OUT_OF_RANGE
severity: Error
source: Reflection
blocking: Save
instance: /Game/Lighting/Sun
property: Intensity
message: Intensity must be between 0 and 100000.
suggested_fix: Clamp value to 100000.
agent_repair: allowed
```

## Severity

Initial severities:

- `Info`: useful state that does not require action.
- `Warning`: suspicious state that may become a problem.
- `Error`: invalid state that blocks at least one workflow.
- `Fatal`: unrecoverable state that prevents safe continuation.

## Blocking Scope

Diagnostics must say what they block when possible:

- Edit.
- Save.
- Play.
- Import.
- Build.
- Bundle.
- Publish.
- Test.

## MCP Relationship

MCP tools should expose diagnostics through commands such as:

```text
diagnostics.list
diagnostics.get
diagnostics.explain
diagnostics.apply_fix
```

Automated fixes must map to editor commands and must respect the command and
semantic change model.

## Consequences

- The editor can show health by project, scene, asset, script, instance, and
  property instead of only streaming logs.
- Agents can troubleshoot by stable diagnostic code and target references.
- Tests can assert exact diagnostic codes for invalid states.
- Builds and CI can fail with actionable messages.
- Safe repairs can be automated without hiding judgment-heavy decisions.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Use log strings as the only error reporting mechanism.
  - Rejected because logs are chronological and too weak for current project
    health, repair, and testing.
- Use ad hoc error enums per subsystem with no common shape.
  - Rejected because editor, MCP, tests, and builds need shared reporting.
- Allow agents to infer repairs from free-form text only.
  - Rejected because repairability must be explicit and permissioned.

## Reopen Conditions

- Stable diagnostic codes become too hard to maintain.
- Runtime diagnostics need a materially different model from editor/project
  diagnostics.
- Collaboration or CI workflows require a richer health-state model.
- A better diagnostics architecture emerges with clear migration path.
