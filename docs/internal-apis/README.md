# Internal API Contract Specs

These M5 specs define internal contracts before runtime, editor, MCP, project,
diagnostics, resource, command, scene, and reflection implementation starts.

They are not implementation APIs yet. They are constraints for later issues so
agents do not invent private mutation paths or cross crate boundaries during
feature work.

## Specs

- `project-editor-session.md`: `Project` and `EditorSession` ownership.
- `scene-reflection.md`: scene graph and reflection metadata contracts.
- `diagnostics-store.md`: current health diagnostics storage and querying.
- `command-change-record.md`: validated commands, change records, undo, and
  dirty-state explanation.
- `runtime-frame.md`: runtime world identity and frame scheduling.
- `signal-bus.md`: deterministic signal descriptors, connections, event queues,
  and frame flush integration.
- `resource-database.md`: resource database over manifests and import cache
  contracts.
- `mcp-command-surface.md`: editor-owned MCP command surface shape.

## Common Rules

- Runtime crates must not depend on editor or MCP implementation crates.
- Editor UI and MCP must mutate project state through command contracts.
- Third-party dependency types must stay behind the crate boundary approved in
  M4 dependency proposals.
- Serialized source file changes require dedicated serialization issues and
  golden tests.
- Public API changes require focused implementation issues and review.
- Unsafe Rust remains forbidden unless an ADR 0017 exception is approved.
