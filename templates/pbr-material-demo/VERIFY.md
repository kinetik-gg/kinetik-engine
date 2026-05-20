# PBR Material Demo Verification

## Required Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Headless/MCP Checks

- `cargo test -p kinetik-editor pbr_material_demo_template_loads_saves_and_runs_headless_smoke`
- `cargo test -p kinetik-render extraction_collects_authored_pbr_material_properties`
- `cargo test -p kinetik-render smoke_image_reflects_pbr_material_factors`

These checks load the template, compare deterministic saved output, extract
authored material factors, and produce nonblank smoke-render output without
requiring a display.

## Screenshot Contract

Current automated editor screenshot comparison is not available. For M29, the
visual smoke evidence is a deterministic headless render that varies by material
color and factors.

Expected future screenshot contract:

- View: `Camera` in `/Game/Workspace`.
- Size: 1280x720.
- Content: material swatches arranged from dielectric rough through metallic
  smooth, with one camera and one light.
- Tolerance: to be defined when editor viewport screenshot comparison exists.

## Manual Review

- Confirm the template lives under `templates/pbr-material-demo/`.
- Confirm the README does not claim texture decoding, imported meshes, normal
  maps, emissive maps, shadows, IBL, HDR, or interactive viewport rendering.
- Confirm no `.kinetik/`, build output, machine-local paths, or generated caches
  are committed.
