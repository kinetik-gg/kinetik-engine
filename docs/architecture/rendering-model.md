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
