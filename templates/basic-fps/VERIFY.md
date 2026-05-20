# Basic FPS Verification

## Required Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Headless Checks

- `cargo test -p kinetik-editor basic_fps_template_loads_saves_and_runs_headless_objective_smoke`
- `cargo test -p kinetik-physics`

The template smoke loads and round-trips the project, runs editor play mode,
simulates first-person input through static collision, raycasts the objective,
marks completion, restarts, and verifies state is reset.

## Manual Review

- Confirm the template lives under `templates/basic-fps/`.
- Confirm scene objects include PlayerStart, Camera, KeyLight, Floor, Wall, and
  Objective.
- Confirm documentation does not claim platform input, mouse capture, scripting
  gameplay, dynamic physics, or interactive viewport rendering.
