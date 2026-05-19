# Kinetik Engine

Kinetik is a Rust-native, Luau-scripted, instance-based game engine with an editor-centered creator workflow.

Core direction:

- Rust core.
- Luau scripting.
- wgpu renderer.
- Vello-powered editor long-term.
- Rapier physics.
- Instance-based scene model.
- File-based, Git-friendly project workspaces scaffolded by Kinetik Studio.
- Agent-executable architecture and development process.

This repository is intentionally scaffolded as a marathon foundation, not a sprint prototype.

Start with:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

See `docs/constitution.md` and `AGENTS.md` before assigning AI agents.
