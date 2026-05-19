# ADR 0014: Editor Command and Semantic Change Model

## Status

Accepted as initial project direction.

## Context

Text-first files help Kinetik work with Git, agents, and external tools, but raw
text diffs are not enough. Large scenes, prefabs, asset manifests, and package
overrides need semantic edits, explainable dirty state, structured diagnostics,
undo/redo, and future semantic merge support.

Unity, Unreal, and Roblox all show the same failure mode in different forms:
when editor state, serialized state, collaboration state, and automation state
drift apart, developers lose trust in the editor.

## Decision

All meaningful editor mutations go through explicit editor commands.

Commands are the shared mutation surface for:

- Editor UI actions.
- MCP mutating tools.
- Undo and redo.
- Dirty-state tracking.
- Validation.
- Diagnostics.
- Semantic diff and future merge tooling.

Each mutating command must produce a structured change record that describes
what changed in semantic terms.

Example command families:

```text
CreateInstance
DeleteInstance
RenameInstance
ReparentInstance
DuplicateInstance
SetProperty
AttachScript
DetachScript
CreatePrefab
ApplyPrefabOverride
RevertPrefabOverride
UpdatePrefabSource
ImportAsset
ChangeImportSetting
```

Example semantic change:

```text
/Game/Workspace/Enemy
  Transform.Position changed [0, 0, 0] -> [4, 0, 2]
  Script EnemyAI attached
```

## Command Rules

- Commands must be validated before mutation.
- Commands must be undoable unless explicitly transient or read-only.
- Commands must identify affected instances, assets, scripts, properties, and
  files through stable IDs plus human-readable paths where possible.
- Commands must report meaningful failures.
- Commands must group related low-level edits into one user-facing undo group.
- Commands must not silently change unrelated project state.
- Commands must be deterministic when serialized as change records.

## Dirty State

Dirty state is derived from saved snapshots and structured change records, not
from hidden editor flags.

The editor should be able to explain why something is dirty:

```text
Scene dirty: /Game/Lighting/DirectionalLight.Intensity changed.
Asset manifest dirty: res://assets/models/tree.glb import scale changed.
Prefab dirty: Enemy.ktprefab has unapplied Transform.Position override.
```

## Semantic Diff and Merge

Kinetik source files remain the Git source of truth. Semantic diff and merge
tooling should operate above raw text by comparing stable IDs, property paths,
instance hierarchy operations, asset GUIDs, and prefab override records.

The initial implementation does not need a full merge tool, but commands and
serialization must preserve enough intent for one.

## MCP Relationship

Mutating MCP tools map directly to editor commands. MCP must not create a second
mutation path.

This keeps human UI actions and agent actions consistent for validation,
undo/redo, diagnostics, dirty state, and saved files.

## Consequences

- Editor features must define commands instead of directly mutating project
  state from UI handlers.
- Undo/redo becomes architectural rather than a late UI layer.
- Agent actions become reviewable and explainable.
- Future semantic merge tools can reason about intent instead of raw line
  positions.
- Tests can assert command results, change records, dirty state, and
  diagnostics together.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Let UI, MCP, importers, and tools mutate project state directly.
  - Rejected because it creates inconsistent validation, dirty state, and
    undo/redo behavior.
- Rely only on Git text diffs.
  - Rejected because scene, prefab, and manifest changes need semantic
    explanation for humans and agents.
- Add semantic merge later without command records.
  - Rejected because merge tooling needs stable intent and identity data from
    the beginning.

## Reopen Conditions

- Command records cannot express required editor operations.
- Performance requirements make command/change-record capture too expensive.
- Collaboration requirements need a different operation model.
- A better semantic merge architecture emerges with clear migration path.
