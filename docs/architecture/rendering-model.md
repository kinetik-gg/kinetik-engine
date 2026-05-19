# Rendering Model

Renderer principle:

> The engine owns rendering structure. Users own material expression.

Initial renderer:

- wgpu.
- Simple forward subset first, without blocking Forward+/clustered lighting.
- Practical metallic/roughness PBR.
- StandardMaterial.
- Render graph direction.
- HDR target, tone mapping, gamma correctness.
- Shadows, environment lighting, and image-based lighting direction.
- Shader graph later compiles to WGSL surface functions.
- Runtime scene state and GPU render state remain separated by render extraction.
- The editor viewport should use the runtime renderer path where practical.
- Rendering failures produce structured diagnostics.

Shader graph pipeline:

```text
Material Graph Editor -> Material Graph IR -> Generated WGSL Surface Function -> Standard PBR Renderer
```

## Editor UI Rendering Fallback

ADR 0001 accepts Vello as the editor UI direction, but Vello maturity must not
force editor model or runtime architecture decisions.

Before implementation of the editor UI milestone, confirm whether Vello is ready
for the required panels, viewport overlays, text quality, input handling, and
platform support. If it is not ready, use a focused fallback decision issue
instead of pushing Vello-specific assumptions into editor state.

Fallback candidates may include immediate-mode or retained UI toolkits, but the
editor data model must remain toolkit-independent. Editor selection, commands,
diagnostics, dirty state, and play mode state belong to engine/editor session
contracts, not to a UI renderer.
