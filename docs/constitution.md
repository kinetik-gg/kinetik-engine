# Kinetik Constitution

## Identity

Kinetik is a Rust-native, Luau-scripted, instance-based game engine with an editor-centered creator-friendly workflow.

Kinetik borrows organizational clarity from Roblox Studio, scene intuition from Godot, visual authoring ambition from Blender, and runtime discipline from Rust.

Kinetik is not a Roblox hosted-server replacement. It does not imply client/server services by default.

## Core Decisions

- Core language: Rust.
- Scripting language: Luau.
- Renderer API: wgpu.
- Shader language: WGSL generated/managed by the engine.
- Editor UI: Vello long-term.
- Physics: Rapier.
- Window/input: winit.
- Audio: Kira preferred.
- Project files: text-first, deterministic, GUID-backed, Git-friendly, and editor-scaffolded.
- Runtime content: generated `.ktbundle` packages.

## Architecture Laws

1. Runtime must not depend on editor.
2. Scene instances are the user-facing authoring model.
3. ECS-like internals may exist but must not leak into normal authoring unless explicitly designed.
4. Luau scripts never hold raw Rust memory, renderer internals, physics internals, or OS capabilities.
5. Serialization is sacred: version changes require ADRs, migrations, and tests.
6. Source assets are project-owned; generated caches and builds are disposable.
7. Engine owns rendering structure; users own material expression through shader graphs/surface shaders.
8. Physics authoring is instance-based; simulation is system-owned.
9. Terrain appears as a Workspace object but may be internally managed by terrain systems.
10. Autonomous agents may execute scoped work, but architectural changes require supervision.
11. Unsafe Rust is forbidden by default; exceptions require an ADR, isolated boundary, documented invariants, tests, and human approval.
12. Dependencies require explicit approval, documented rationale, boundary ownership, and license/safety review.
13. Play mode runs on sandboxed runtime state; runtime changes persist only through explicit editor commands.

## Code Quality Laws

- No god files.
- No god objects.
- No vague utility modules.
- Avoid 3+ nesting.
- Use typed IDs, not interchangeable integers.
- Prefer concrete types until abstraction pressure exists.
- Public APIs require documentation and tests.
- Errors must explain failure.
- Unsafe must never be used as a shortcut around ownership, lifetimes, API design, or compiler errors.
- External dependency types should not leak into public Kinetik APIs unless deliberately approved.
