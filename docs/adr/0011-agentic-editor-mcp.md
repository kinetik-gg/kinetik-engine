# ADR 0011: Agentic Editor MCP Contract

## Status

Accepted as initial project direction.

## Context

Kinetik is intended to support agent-executable game development. Agents should
be able to perform mechanical editor and runtime tasks such as creating instances,
editing properties, importing assets, attaching scripts, running scenes, reading
diagnostics, and troubleshooting broken project state.

Direct file edits are not enough once the editor is open. They bypass undo,
validation, selection, diagnostics, dirty-state tracking, importer state, and
runtime/editor mode boundaries.

## Decision

Kinetik Studio will own an MCP server that exposes explicit, validated,
undoable editor commands for agentic game-development workflows.

Agents may inspect and mutate projects through the MCP contract, but the editor
remains the source of truth for project state, scene state, validation,
serialization, asset import state, command history, and play mode boundaries.

Human review remains required for gameplay feel, visual judgment, product
direction, public API changes, serialized format changes, dependency additions,
and broad architectural changes.

## Consequences

- Editor commands must be designed as stable automation surfaces, not only UI
  button handlers.
- Scene instances need stable, agent-readable addressing through GUIDs, runtime
  typed IDs, and human-readable paths.
- Reflection metadata must support safe property inspection and editing.
- Diagnostics must be structured enough for agents to identify and repair
  repeated classes of problems.
- The editor/runtime boundary remains intact: runtime crates do not depend on
  editor or MCP server implementation details.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Allow agents to edit project files directly.
  - Rejected for open-editor workflows because it bypasses validation, undo,
    diagnostics, importer state, and editor dirty-state tracking.
- Expose only UI automation.
  - Rejected because UI automation is useful for verification but too brittle
    for core mechanical editing.
- Expose a broad shell-like automation surface.
  - Rejected because project-local, validated editor commands are safer and
    easier to reason about.

## Reopen Conditions

- MCP cannot express required editor workflows cleanly.
- A better agent protocol emerges with clear migration path.
- Security or permission requirements invalidate the initial authority model.
- Editor architecture changes make a separate automation server preferable.
