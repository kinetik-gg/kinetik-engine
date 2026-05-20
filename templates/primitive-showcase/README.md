# Primitive Showcase

## Purpose

Primitive Showcase proves Kinetik can preserve a simple authored 3D scene made
of primitive instances, camera, lighting, transforms, renderer extraction, and
play-mode smoke checks.

## Acceptance Target

M27: Primitive Showcase Template, issue #177.

## Engine Features

- Project-shaped first-party template layout.
- Default scene hierarchy with authored Workspace content.
- Camera3D, Light3D, and visible Part instances.
- Deterministic transform data and save/reload behavior.
- Renderer primitive extraction with StandardMaterial fallback.
- Headless smoke render output.
- Editor play-mode start, step, and stop lifecycle.

## Not Covered

- Imported meshes or textures.
- Authored material assets beyond the current fallback material scaffold.
- GPU-backed editor viewport rendering.
- Gameplay input, physics, interaction, or FPS controller behavior.

## Human Verification

Review the template hierarchy and verification notes. Current visual evidence is
headless smoke-render output; interactive editor-window viewport rendering is a
follow-up after the window paints renderer output.
