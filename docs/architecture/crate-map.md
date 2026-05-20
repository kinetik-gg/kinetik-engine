# Crate Map

## Foundational

- `kinetik-core`: typed IDs, errors, shared primitives.
- `kinetik-reflect`: class-level property descriptors and reflection metadata.
- `kinetik-test`: test utilities and fixtures.

## Runtime Domains

- `kinetik-scene`: instance hierarchy, transforms, and scene serialization contracts.
- `kinetik-signal`: deterministic signal/event bus.
- `kinetik-resource`: asset paths, handles, metadata, cache/import skeleton, and
  resource reference validation over scene/reflection state.
- `kinetik-script`: runtime-agnostic script lifecycle, attachment,
  diagnostics, provenance, and handle contracts.
- `kinetik-script-luau`: Luau VM bridge and Luau-specific typed API generation
  from reflection metadata.
- `kinetik-render`: wgpu renderer, PBR, material/shader system.
- `kinetik-physics`: Rapier-backed instance-authored physics.
- `kinetik-audio`: buses, playback, spatial audio abstractions.
- `kinetik-ui`: runtime UI model.
- `kinetik-terrain`: terrain data, chunks, tools-facing APIs.
- `kinetik-bundle`: build/load/verify `.knbundle` packages.
- `kinetik-app`: runtime app loop and subsystem orchestration.

## Editor

- `kinetik-editor`: Kinetik Studio shell, panels, inspector, viewport, Vello UI.

Future MCP implementation should live on the editor side of the boundary. Runtime
crates may expose typed state, diagnostics, and reflection metadata, but they must
not depend on editor-owned MCP server code.
