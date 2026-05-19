# Architecture Overview

```text
Kinetik Studio / Editor
  -> Agentic Editor MCP
  -> Kinetik App / Runtime
    -> Scene, Resources, Script, Render, Physics, Audio, UI, Terrain, Bundles
      -> Core primitives, math, errors, handles, serialization
```

The editor uses the runtime model but keeps editor-only state separate. Runtime crates must not depend on editor crates.

Agentic editor automation is exposed through the editor-owned MCP contract in
`docs/architecture/agentic-editor-mcp.md`. Agents operate through validated,
undoable editor commands instead of bypassing the editor with direct project-file
edits.

Editor mutations are modeled as semantic commands and structured change records
so undo/redo, dirty state, diagnostics, MCP operations, and future merge tooling
share one mutation path.

Play mode runs on a sandboxed runtime copy of edit-world state. Runtime changes
do not affect saved project state unless explicitly applied through editor
commands.

The runtime is 3D-first, instance-scripted, deterministic-first, and standalone
from editor crates. Basic outbound HTTP is governed separately as a permissioned
service with script and instance provenance.

Runtime execution uses deterministic frame order, separate variable update and
fixed simulation, deterministic event flush points, and safe structural-change
sync points.
