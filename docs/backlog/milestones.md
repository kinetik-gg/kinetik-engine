# Kinetik Milestones

This roadmap is the local planning source. GitHub issues are the executable
backlog. Each implementation issue should map to one branch/worktree and one
focused patch.

## Product Target: First Playable 3D Templates

The first public-facing proof point is a small set of polished 3D templates
that demonstrate the engine and editor working together end to end:

- Basic primitive showcase.
- PBR material demo scene.
- Basic FPS prototype.

These templates are acceptance targets, not side demos. Roadmap work should
prioritize capabilities that make these projects authorable, inspectable,
playable, diagnosable, and eventually packageable through Kinetik workflows.

Near-term template work is 3D-first. 2D templates, multiplayer, AI combat,
inventory systems, networking, and large content production are out of scope
until the first 3D template set is proven.

Relevant accepted ADRs:

- ADR 0006: Renderer and Shader Graph.
- ADR 0019: Edit, Play, and Runtime State Boundaries.
- ADR 0020: Runtime Execution Model.

## M1: Core Foundation

Goal: establish the shared primitives that every later crate depends on.

Key outputs:

- Typed ID and GUID policy.
- Shared error and diagnostic primitives.
- Core value primitives needed by reflection and scenes.
- Reflection descriptor foundation.
- Baseline test fixtures.

Representative issues:

- Core typed ID invariants.
- Core diagnostic and error foundation.
- Core math/value primitives.
- Reflection descriptor model.
- Reflection value container.
- Test fixture crate.

## M2: Scene and Instance Core

Goal: build the deterministic in-memory scene and instance model.

Key outputs:

- Root `Game` instance.
- Default service instances.
- Instance class registry scaffold.
- Parent/child hierarchy.
- GUID/runtime ID/path lookup.
- Reflected property storage and validation.
- Structural mutation APIs.

Representative issues:

- Scene instance class registry scaffold.
- Scene hierarchy model.
- Default scene scaffold.
- Instance property storage.
- Scene structural mutation queue.

## M3: Project Serialization and Assets

Goal: make Kinetik projects scaffoldable, loadable, and Git-friendly.

Key outputs:

- Project layout model.
- `Kinetik.toml` contract.
- `assets.ktmanifest` in-memory model.
- Asset identity and `res://` references.
- TOML/RON dependency proposal.
- Deterministic scene and manifest serialization.
- Golden fixtures.

Representative issues:

- Resource asset identity model.
- Resource manifest in-memory model.
- Serialization dependency proposal.
- Project layout scaffold model.
- Scene serialization contracts.
- Asset manifest serialization contracts.

## M4: Commands and Runtime Kernel

Goal: establish the mutation and execution spine before editor/MCP work.

Key outputs:

- Deterministic signal delivery.
- Editor command/change-record primitives.
- Dirty-state explanation.
- Runtime frame-step skeleton.
- Edit/play world boundary model.
- Script lifecycle scheduler contract.

Representative issues:

- Signal bus deterministic delivery.
- Editor command core.
- Runtime frame step skeleton.
- Edit/play world boundary model.
- Script lifecycle scheduler contract.

## M5: Luau Scripting Slice

Goal: prove instance-scripted gameplay with familiar Luau-facing APIs.

Key outputs:

- Dependency proposal for Luau integration.
- `Ready`, `Update`, `PhysicsUpdate`, and `Exit` dispatch.
- Safe instance handles.
- Reflected property access.
- Script diagnostics.
- Queued structural changes from scripts.

Representative issues:

- Luau dependency proposal.
- Luau bridge scaffold.
- Script lifecycle dispatch.
- Safe instance handle API.
- Script property access diagnostics.

## M6: Editor Command Surface

Goal: expose project mutation through validated editor commands.

Key outputs:

- Create/delete/rename/reparent instance commands.
- Set property command.
- Undo/redo groups.
- Semantic change records.
- Dirty-state explanations.
- Command diagnostics.

Representative issues:

- Command result and validation model.
- Instance mutation commands.
- Property mutation command.
- Undo/redo command grouping.
- Dirty-state explanation tests.

## M7: MCP Read-Only Automation

Goal: let agents inspect project, scene, runtime, and diagnostics state through
semantic tools.

Key outputs:

- Editor-owned MCP server scaffold.
- `project.open` / `project.create_temp` shape.
- `scene.list_instances`.
- `scene.get_instance`.
- `diagnostics.list`.
- `editor.get_dirty_state`.
- Test harness hooks.

Representative issues:

- MCP server dependency proposal.
- MCP read-only command schema.
- MCP diagnostics listing.
- MCP scene inspection.
- MCP project temp workspace test support.

## M8: MCP Mutating Automation

Goal: let agents safely edit through the same command path as the editor.

Key outputs:

- `scene.create_instance`.
- `scene.set_property`.
- `scene.reparent_instance`.
- `editor.undo` / `editor.redo`.
- Mutating MCP commands mapped to editor commands.
- Dirty-state and diagnostics verification.

Representative issues:

- MCP create instance command.
- MCP set property command.
- MCP undo/redo command.
- MCP command-to-change-record integration tests.

## M9: First Editor Shell

Goal: create the first visible Kinetik Studio loop.

Key outputs:

- Window shell.
- Explorer panel.
- Inspector panel.
- Diagnostics panel.
- Basic viewport placeholder.
- Manual/screenshot smoke verification.

Representative issues:

- Editor shell dependency proposal.
- Window and app shell.
- Explorer displays default scene hierarchy.
- Inspector reads reflected properties.
- Diagnostics panel displays structured diagnostics.

## M10: First 3D Authoring Slice

Goal: prove the smallest end-to-end 3D game-development workflow that can grow
into the primitive showcase template.

Key outputs:

- Create/scaffold project.
- Load scene.
- Add primitive 3D instances.
- Set transform and material-facing reflected properties.
- Save and reload.
- Enter play mode.
- Run deterministic frame steps.
- Inspect diagnostics.
- Capture enough editor/runtime state through MCP to verify the slice
  semantically.

Representative issues:

- Hello scene upgrade to a 3D primitive scene.
- End-to-end project scaffold/load/save test.
- First play-mode smoke.
- MCP-driven vertical slice smoke.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Golden project/scene fixture once serialization is active.
- MCP or headless smoke for create, save, reload, play, step, and diagnostics.

Human verification:

- Confirm the editor can open the project, show the primitive scene, enter play
  mode, and stop without leaking play-state mutations into edit state.

## M11: 3D Rendering Foundation

Goal: make runtime and editor rendering capable enough to support the first
template scenes without locking Kinetik into a toy renderer.

Key outputs:

- Runtime renderer path used by the editor viewport where practical.
- Camera and light instances sufficient for basic 3D scenes.
- Built-in primitive mesh resources for cube, sphere, capsule, cylinder, cone,
  plane, and quad.
- Initial mesh/material extraction boundary from scene state to render state.
- PBR-compatible `StandardMaterial` scaffold with safe fallback materials.
- Structured render diagnostics for missing cameras, meshes, materials,
  shaders, textures, and lights.

Representative issues:

- Renderer dependency and crate-boundary confirmation.
- Built-in primitive mesh resource set.
- Camera and light instance render extraction.
- `StandardMaterial` reflected property scaffold.
- Safe fallback material and missing-resource diagnostics.
- Editor viewport renders through runtime renderer path.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Unit tests for deterministic primitive resource identifiers and material
  defaults.
- Integration or fixture tests for render extraction diagnostics.
- Screenshot smoke once viewport rendering exists.

Human verification:

- Confirm a primitive scene renders with stable camera framing, usable
  selection context, and meaningful diagnostics when resources are invalid.

## M12: Primitive Showcase Template

Goal: ship the first Kinetik template proving that basic 3D authoring,
hierarchy, transforms, materials, viewport inspection, and play-mode smoke work
as a coherent editor/runtime loop.

Key outputs:

- `templates/primitive_showcase` project once template packaging exists, or an
  equivalent tracked example until the template directory contract is approved.
- A curated scene containing all supported built-in primitive meshes.
- Clear hierarchy, names, transforms, and material assignments.
- Camera and lighting setup suitable for immediate inspection.
- Save/reload fixture or golden output for the authored scene.
- Verification notes describing what the template proves and known limitations.

Representative issues:

- Primitive showcase template contract.
- Primitive showcase scene authoring fixture.
- Primitive transform/material editing smoke.
- Primitive showcase save/reload golden.
- Primitive showcase editor screenshot verification.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Golden fixture for the template scene once serialization is active.
- MCP or headless smoke for open, inspect hierarchy, enter play mode, step, and
  stop.

Human verification:

- Confirm the scene is visually legible, primitives are recognizable, editor
  selection/inspection is usable, and play mode does not persist temporary
  runtime mutations.

## M13: PBR Material Demo Scene

Goal: demonstrate Kinetik's practical PBR direction with a compact scene that
exercises material, lighting, texture, import, and render-diagnostic workflows.

Key outputs:

- `templates/pbr_material_demo` project once template packaging exists, or an
  equivalent tracked example until the template directory contract is approved.
- Metallic/roughness material range display.
- Normal map and emissive material examples when supported.
- Directional and local light examples matching the current renderer stage.
- Imported mesh and texture path through the asset manifest once import exists.
- Render diagnostics for missing or invalid material/texture inputs.
- Screenshot or golden visual references once rendering is stable enough.

Representative issues:

- PBR demo material range fixture.
- Texture and material asset import smoke.
- PBR light setup and diagnostics.
- Imported mesh render smoke.
- PBR demo screenshot verification.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Unit tests for material defaults and validation.
- Integration/golden tests for material and texture references once asset
  serialization is active.
- Screenshot smoke for expected material and lighting visibility.

Human verification:

- Confirm the scene communicates the current material model honestly: no
  overpromising unsupported shadows, IBL, graph features, or post-processing.

## M14: Basic FPS Prototype

Goal: prove Kinetik can author and run a minimal playable 3D game loop, not just
render static scenes.

Key outputs:

- `templates/basic_fps` project once template packaging exists, or an equivalent
  tracked example until the template directory contract is approved.
- First-person camera/controller using approved input and runtime boundaries.
- Static collision against simple level geometry.
- Raycast or proximity interaction for a simple objective.
- Minimal game loop: start, move/look, interact or collect, open/complete goal,
  restart.
- Play-mode diagnostics and no persistence of runtime-only state unless
  explicitly applied through editor commands.
- Human playtest checklist for feel, camera comfort, and interaction clarity.

Representative issues:

- FPS template task contract and control scheme.
- First-person controller runtime slice.
- Static collision smoke for primitive level geometry.
- Interaction/raycast objective slice.
- FPS play-mode diagnostics and restart smoke.
- FPS human playtest checklist.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Unit tests for deterministic controller math where practical.
- Integration/headless tests for collision, objective state, restart, and
  play-mode lifecycle.
- MCP play smoke for start, step, diagnostics, and stop once available.

Human verification:

- Confirm movement, mouse look, interaction, and restart feel acceptable for a
  prototype before treating the template as complete.
