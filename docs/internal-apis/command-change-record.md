# Command and ChangeRecord Contract

## Purpose

Define the shared mutation surface for editor UI, MCP, dirty-state tracking,
undo/redo, validation, diagnostics, tests, and future semantic diff/merge.

## Owning Crates

- Future command owner crate or module: command input/result types,
  validation, command execution, change records, undo records, redo records, and
  dirty-state explanation.
- `kinetik-editor`: command dispatch, command history UI state, active undo
  groups, and presentation.
- Domain crates: validate domain invariants and apply domain mutations through
  command-owned transactions.
- MCP implementation: maps mutating tools to commands.

## Command Contract

Commands are explicit, validated, and deterministic. A command must:

- Identify its target mode: edit or play where ambiguity exists.
- Validate before mutation.
- Return structured success or failure.
- Return diagnostics for rejected input.
- Produce one or more semantic change records on success.
- Be undoable unless explicitly transient or read-only.
- Avoid silent mutation of unrelated project state.

Initial command families include scene instance mutations, reflected property
writes, script attachment, asset import/reimport, project open/save/reload, and
play control commands where appropriate.

## ChangeRecord Contract

Change records describe semantic intent and affected targets:

- Command kind.
- Stable target IDs and human-readable paths.
- Property paths and old/new values where applicable.
- Asset GUIDs and paths where applicable.
- Script paths where applicable.
- Files/documents affected.
- Undo group ID.
- Dirty-state summary text.

Change records are not raw text diffs. They may later support semantic diff and
merge.

## Dependency Boundaries

No dependency is approved by this contract. Command APIs expose Kinetik-owned
IDs, values, diagnostics, and document handles.

Commands must not expose editor widget, MCP transport, serializer parser, or VM
types.

## Serialized-Format Impact

No serialized-format change is approved here.

If change records are later serialized for history, collaboration, or merge
tooling, that requires a dedicated serialized-format issue and migration plan.

## Diagnostics Behavior

Command failures return diagnostics rather than partial mutations. Diagnostics
should identify target object, property, asset, command kind, and blocking
scope.

Automated repair commands must be normal commands and produce normal change
records.

## Public API Constraints

- Public command APIs must be stable automation surfaces, not UI button handlers.
- Editor UI and MCP must share command execution paths.
- Domain crates may expose validation helpers, but command orchestration owns
  mutation transactions.

## Follow-Up Issues

- M10 command result model.
- M10 semantic change records.
- M10 undo/redo core.
- M13 editor command surface.
- M15 mutating MCP command mapping.
