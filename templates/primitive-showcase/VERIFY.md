# Primitive Showcase Verification

## Required Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Headless/MCP Checks

- `cargo test -p kinetik-editor primitive_showcase_template_loads_saves_and_runs_headless_smoke`
- `cargo test -p kinetik-editor primitive_showcase_play_mode_does_not_persist_runtime_state`

These checks load the template, compare deterministic saved output, extract
render primitives, produce a nonblank smoke image, and run play-mode lifecycle
checks without requiring a display.

## Golden Files

- `Kinetik.toml`
- `project/assets.knmanifest`
- `scenes/main.knscene`

The editor save/reload test writes these documents to a temporary project root
and compares them byte-for-byte against the committed template files.

## Screenshots

Current automated screenshot comparison is not available for the editor window.
For M27, visual evidence is the deterministic nonblank headless smoke image
checked by tests. Renderer-backed editor viewport screenshots are deferred to a
follow-up viewport-rendering issue.

Expected future screenshot contract:

- View: `Camera` in `/Game/Workspace`.
- Size: 1280x720.
- Content: three visible cube primitives, one camera, one light, stable framing.
- Tolerance: to be defined when screenshot comparison exists.

## Manual Review

- Confirm the template lives under `templates/primitive-showcase/`.
- Confirm `README.md` does not overclaim imported assets, authored materials, or
  interactive viewport support.
- Confirm `VERIFY.md` names the current screenshot gap.
- Confirm no `.kinetik/`, build output, machine-local paths, or generated caches
  are committed.

## Known Gaps

- Renderer-backed editor viewport drawing and screenshot automation are not part
  of this template slice.
- Authored material assets and imported mesh/texture paths begin in M28/M29.
