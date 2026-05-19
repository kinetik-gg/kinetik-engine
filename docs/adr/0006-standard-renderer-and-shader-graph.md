# ADR 0006: Renderer and Shader Graph

## Status

Accepted as initial project direction.

## Context

Kinetik is a marathon-scale game engine intended to be built by coordinated
human and AI-agent contributors.

Rendering architecture must not accidentally settle for a toy pipeline. The
implementation may be staged, but early choices must preserve the target engine
direction.

## Decision

Practical PBR plus graph-authored surface shaders.

Kinetik targets a modern `wgpu` 3D renderer with:

- PBR material system.
- Render-graph organization.
- HDR pipeline.
- Tone mapping and color correctness.
- Shadow mapping.
- Environment lighting and image-based lighting.
- Material graph to generated WGSL surface functions.
- A path toward Forward+/clustered lighting.
- Render diagnostics for missing or invalid meshes, materials, shaders,
  textures, cameras, and lights.

The initial implementation may begin with a simple forward-rendered subset, but
must not choose architecture that blocks the target renderer.

The implementation path is staged:

1. Simple forward subset for the first visible scene.
2. PBR-compatible `StandardMaterial`.
3. HDR target, tone mapping, and gamma/color correctness.
4. Shadow and environment lighting.
5. Render graph organization.
6. Material graph IR and generated WGSL surface functions.
7. Forward+/clustered lighting path.

Deferred rendering, global illumination experiments, meshlet-style pipelines,
and deeper GPU-driven rendering remain future options, not initial commitments.

Renderer boundaries:

- Runtime scene state and GPU render state are separate.
- Render extraction builds render data from scene instances.
- Rendering observes coherent world snapshots from the runtime frame lifecycle.
- The editor viewport should use the runtime renderer path where practical.
- Authoring APIs must not expose raw `wgpu` types.
- Users own material expression; Kinetik owns render pipeline structure.

## Consequences

- Kinetik keeps a clear high-end rendering target while allowing incremental
  delivery.
- Early visible-scene work must still fit the future render graph, HDR,
  material graph, and Forward+/clustered lighting direction.
- Render systems need explicit extraction boundaries between runtime instances
  and GPU backend state.
- Rendering errors should surface through structured diagnostics.
- This decision shapes crate boundaries, public APIs, editor workflows, tests,
  and agent assignments.
- Reopening requires a follow-up ADR with alternatives and migration implications.

## Alternatives Considered

- Toy/simple renderer as the long-term target.
  - Rejected because Kinetik needs rendering architecture that can grow into the
    intended engine.
- Deferred renderer first.
  - Rejected for the initial implementation because it increases complexity
    before the editor/runtime loop is proven.
- Expose raw `wgpu` pipeline ownership to normal authoring.
  - Rejected because Kinetik owns rendering structure while users own material
    expression.
- Separate editor viewport renderer.
  - Rejected as the default because it risks divergence from runtime rendering.

## Reopen Conditions

- Technical blocker discovered.
- Better option emerges with clear migration path.
- Product direction changes.
