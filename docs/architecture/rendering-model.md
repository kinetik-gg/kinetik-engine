# Rendering Model

Renderer principle:

> The engine owns rendering structure. Users own material expression.

Initial renderer:

- wgpu.
- Forward renderer first.
- Practical metallic/roughness PBR.
- StandardMaterial.
- HDR target, tone mapping, gamma correctness.
- Shader graph later compiles to WGSL surface functions.

Shader graph pipeline:

```text
Material Graph Editor -> Material Graph IR -> Generated WGSL Surface Function -> Standard PBR Renderer
```
