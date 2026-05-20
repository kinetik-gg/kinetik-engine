# PBR Material Demo

## Purpose

PBR Material Demo proves Kinetik can author and preserve a compact material
range scene using the current `StandardMaterial` scaffold, lighting, renderer
extraction, and headless smoke-render verification.

## Acceptance Target

M29: PBR Material Demo Scene, issue #181.

## Engine Features

- Project-shaped first-party template layout.
- Authored Camera3D, Light3D, and visible Part instances.
- Inline Part material properties for base color, metallic, and roughness.
- Deterministic renderer extraction into `StandardMaterial`.
- Headless smoke render output that changes with material factors.
- Editor load/save and play-mode smoke coverage.

## Not Covered

- Real texture decoding or GPU texture binding.
- Imported glTF/GLB mesh parsing.
- Normal maps, emissive maps, shadows, image-based lighting, or HDR output.
- GPU-backed editor viewport rendering.

## Human Verification

Review the template hierarchy, material values, and verification notes. Current
visual evidence is deterministic nonblank headless smoke-render output; the
interactive editor window still needs a renderer-backed viewport follow-up.
