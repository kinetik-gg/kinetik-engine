# ADR 0016: Prefab and Package Override Model

## Status

Accepted as initial project direction.

## Context

Prefabs and packages are a major source of editor trust problems in existing
engines when local modifications are hidden, source updates overwrite edits, or
the editor reports changed state without explaining why.

Kinetik needs reusable instance trees that stay inspectable, mergeable, and safe
for humans, Git, and MCP agents.

## Decision

Kinetik prefabs are serialized instance trees. Prefab instance customization is
represented as explicit, inspectable, property-path-based override records.

A prefab asset contains:

- Stable GUID.
- Source path.
- Schema/version.
- Root instance tree.
- Optional exposed-property metadata in the future.

A prefab instance contains:

- Source prefab GUID and path.
- Source prefab version or content hash.
- Local instance GUID.
- Explicit override records.

Override records are first-class data, not accidental differences.

Example override records:

```text
OverrideProperty(
  instance_path: "Enemy/Visuals",
  property: "Transform.Scale",
  value: [1.2, 1.2, 1.2],
)

OverrideAddedChild(
  parent_path: "Enemy",
  child_guid: "...",
)

OverrideRemovedChild(
  instance_path: "Enemy/DefaultWeapon",
)
```

## Update Rules

Updating a prefab source must preserve local overrides unless the override target
no longer exists or becomes incompatible.

When an override target disappears, changes class, or rejects a value, Kinetik
emits structured diagnostics instead of guessing.

Prefab modified state must be explainable:

```text
Enemy prefab instance has 3 overrides:
- Transform.Position
- Health.Max
- Added child: LootDrop
```

## MCP Relationship

MCP should expose prefab operations through editor commands:

```text
prefab.list_overrides
prefab.apply_override
prefab.revert_override
prefab.update_source
prefab.unpack
```

MCP must not mutate prefab override state outside the command and semantic
change model.

## Package Relationship

Packages may contain prefabs, assets, scripts, scenes, or bundles. Package
updates must follow the same principle: local project changes are explicit,
inspectable, and preserved unless a diagnostic explains why they cannot be.

The initial prefab override model should be designed so packages can reuse it
rather than inventing a separate override mechanism.

## Consequences

- Prefab customization is visible in the editor and reviewable in source files.
- Users and agents can understand why a prefab instance is modified.
- Prefab source updates do not silently erase local edits.
- Diagnostics can report broken or incompatible overrides with stable targets.
- Semantic diff and merge can reason about prefab intent rather than raw text
  changes.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Treat prefab instances as deep copies after instantiation.
  - Rejected because updates and source relationships become unclear.
- Track prefab changes as implicit differences only.
  - Rejected because modified state becomes difficult to inspect, merge, and
    repair.
- Allow source updates to overwrite local edits by default.
  - Rejected because it destroys user trust and makes package workflows risky.
- Build a separate package override system.
  - Rejected for the initial direction because packages should reuse the prefab
    override principles when they contain instance trees.

## Reopen Conditions

- Explicit override records cannot express nested prefab workflows.
- Package requirements force a broader override system.
- Semantic merge requirements need a different representation.
- Runtime loading requirements make override application too expensive.
