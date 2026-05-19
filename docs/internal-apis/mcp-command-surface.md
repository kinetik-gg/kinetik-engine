# MCP Internal Command Surface Contract

## Purpose

Define the editor-owned semantic automation surface before MCP read-only and
mutating implementations begin.

## Owning Crates

- `kinetik-editor`: MCP server host, editor session authority, command dispatch,
  selection/focus state, active project/scene state, play-mode ownership, and
  dirty-state reporting.
- Domain crates: provide engine-owned state, validation, diagnostics, and
  command inputs/results.
- Runtime crates: expose inspectable runtime state without depending on MCP.

## Command Surface Contract

MCP commands are semantic editor operations, not UI automation and not direct
file writes when the editor is open.

Read-only commands should cover:

- Project status and active documents.
- Scene hierarchy and instance details.
- Reflected property descriptors and values.
- Resource lookup and validation state.
- Diagnostics listing/explanation.
- Dirty-state explanation.
- Selection and active editor state.
- Play/runtime state inspection when play world exists.

Mutating commands must map to command/change-record contracts:

- Create/delete/rename/reparent/duplicate instance.
- Set reflected property.
- Attach/detach script.
- Import/reimport asset.
- Undo/redo.
- Play/stop/step.
- Apply approved diagnostic fix.

Mutating commands must declare or imply target mode. Ambiguous edit/play
commands fail with diagnostics.

## Dependency Boundaries

MCP transport/server dependencies are not approved by this contract. They need a
future M14 dependency proposal or installation issue.

MCP schemas expose Kinetik-owned IDs, paths, values, diagnostics, command
results, and undo group IDs. They do not expose editor widget types or runtime
memory addresses.

## Serialized-Format Impact

No serialized-format impact is approved.

If MCP command schemas are persisted or versioned as generated artifacts, create
a focused generated-format issue.

## Diagnostics Behavior

Every MCP command returns success/failure and relevant diagnostics. Failure
responses should identify:

- Command name.
- Target mode.
- Target object/resource/property.
- Stable diagnostic codes.
- Suggested safe next action where available.

Repair commands must go through editor commands.

## Public API Constraints

- MCP is editor-owned. Runtime crates must not depend on MCP server code.
- MCP cannot bypass validation, command history, dirty-state tracking, undo, or
  play/edit boundaries.
- MCP cannot expose arbitrary shell, OS, network, or filesystem authority.

## Follow-Up Issues

- M14 MCP server dependency proposal.
- M14 read-only command schema.
- M15 mutating command mapping.
- M21 MCP/editor parity tests.
