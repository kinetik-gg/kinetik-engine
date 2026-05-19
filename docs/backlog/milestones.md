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

## Planning Corrections

The roadmap must not jump from early scene/project scaffolds into editor or
template work without the engine substrate those workflows need.

Before deeper runtime, editor, MCP, or template implementation, agents must have
enough written contract to avoid inventing APIs during feature work:

- Dependency-backed systems need dependency surveys or proposals before crates
  are added.
- Architecture-sensitive implementation needs internal API specs before code
  starts.
- Editor work must consume project, scene, reflection, command, diagnostics,
  resource, and runtime APIs instead of creating private editor-only behavior.
- Play mode must use a runtime sandbox derived from edit state, not mutate saved
  project state directly.
- Template milestones are acceptance targets. They should not pull unplanned
  engine/editor behavior into narrow content patches.

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

## M4: Engine Dependency Surveys

Goal: review dependency choices deeply enough that later engine and editor work
has explicit crate ownership, safety boundaries, and public API constraints.

Key outputs:

- Serialization dependency proposal completed or updated for `serde`, TOML, and
  RON.
- Core utility survey covering math, durable identity, arenas/slot maps, error
  helpers, hashing, and whether external types may appear in public APIs.
- Runtime/app survey covering event loop, time/clock, input boundary, and async
  policy.
- Renderer survey covering `wgpu`, shader tooling, image/texture loading, and
  render API boundaries.
- Editor/window/UI survey covering `winit`, Vello, text/layout, accessibility,
  and screenshot/test harness needs.
- Physics survey covering Rapier ownership and Kinetik-facing physics types.
- Asset import survey covering glTF/GLB, image formats, content hashing,
  importer versioning, and deterministic cache keys.
- Luau survey covering embedding strategy, VM ownership, sandboxing, typed
  bindings, and generated definitions.

Representative issues:

- Core dependency survey.
- Runtime/app dependency survey.
- Renderer dependency proposal.
- Editor/window/UI dependency proposal.
- Physics dependency proposal.
- Asset import dependency proposal.
- Luau dependency proposal.

Implementation level: Level 0.

Required tests/checks:

- Relevant docs/ADR consistency checks.
- `cargo fmt --check` if dependency proposal files or generated tables are
  touched.

Human verification:

- Approve dependency additions separately before implementation issues install
  crates.

## M5: Internal API Contract Specs

Goal: define internal contracts before runtime, editor, MCP, and template agents
depend on behavior that has not been designed.

Key outputs:

- `Project` API spec for create/open/save, project metadata, active scenes,
  manifests, diagnostics, and path policy.
- `Scene` API spec for identity, class registry, hierarchy, properties,
  structural changes, serialized document conversion, and validation.
- `Reflection` API spec for descriptors, validation, editor hints,
  serialization mapping, scriptability, and play-mode mutability.
- `DiagnosticsStore` API spec for current-health diagnostics, logs, filtering,
  blocking scopes, repairability, and lifecycle.
- `Command` / `ChangeRecord` API spec for validation, execution, undo/redo,
  dirty-state explanation, and semantic diffs.
- `RuntimeWorld` API spec for edit-world cloning, play-world identity,
  runtime-only spawned instances, and teardown.
- `FrameScheduler` API spec for variable update, fixed update, deterministic
  event flush points, and safe structural-change sync points.
- `ResourceDatabase` API spec for manifest lookup, resource references, missing
  asset diagnostics, import cache state, and dependency lookup.
- `EditorSession` API spec for active project, active scene, selection, panels,
  document dirty state, and mode ownership.
- MCP internal command surface spec mapping editor automation to command APIs,
  not UI automation.

Representative issues:

- Project and editor session internal API spec.
- Scene/reflection/serialization internal API spec.
- Diagnostics store internal API spec.
- Command/change-record internal API spec.
- Runtime world/frame scheduler internal API spec.
- Resource database/import state internal API spec.
- MCP internal command surface spec.

Implementation level: Level 0.

Required tests/checks:

- Relevant docs/ADR consistency checks.
- Verify each spec names owning crates and dependency boundaries.

Human verification:

- Confirm specs are sufficient for independent implementation agents to proceed
  without inventing public APIs.

## M6: Project Model and Diagnostics Store

Goal: create the engine-owned project/document health layer that editor,
serialization, MCP, and tests can share.

Key outputs:

- Project identity/settings model for `Kinetik.toml`.
- Project layout validation wired to structured diagnostics.
- Active scene/document references without editor-only state.
- Diagnostics store for current project health, with stable codes and blocking
  scopes.
- Separation between diagnostics and chronological logs.
- Project-level test fixtures.

Representative issues:

- Project model scaffold.
- Project diagnostics store.
- Project layout validation diagnostics.
- Project fixture helpers.
- Project metadata contract tests.

Implementation level: Level 2.

Required tests/checks:

- Level 2 checks from `AGENTS.md`.
- Unit tests for project layout validation and diagnostic filtering.
- Golden fixtures once project serialization is active.

Human verification:

- Confirm the project model does not depend on editor crates or UI state.

## M7: Engine Class and Spatial Model

Goal: deepen the instance model enough for real 3D authoring without coupling it
to rendering, physics, or editor UI.

Key outputs:

- Built-in class descriptors beyond root services, including initial 3D-facing
  instance classes.
- Clear class capability metadata or inheritance/composition policy.
- Transform property contract for local transforms.
- World-transform derivation policy and deterministic traversal.
- Transform dirty/update behavior.
- Initial bounds/AABB contract for selection, picking, physics, and render
  extraction.
- Validation diagnostics for invalid class/property/spatial state.

Representative issues:

- Built-in 3D class descriptor set.
- Transform property and validation contract.
- World transform derivation.
- Spatial bounds contract.
- Class capability metadata.

Implementation level: Level 2.

Required tests/checks:

- Level 2 checks from `AGENTS.md`.
- Determinism tests for traversal and transform derivation.
- Invalid class/property diagnostics tests.

Human verification:

- Confirm class names and property paths match the accepted reflection and Luau
  naming direction.

## M8: Runtime World and Frame Kernel

Goal: establish the deterministic runtime world and frame-step skeleton before
script, physics, render, play mode, or MCP runtime inspection depend on it.

Key outputs:

- Runtime world derived from an edit scene/document.
- Runtime IDs distinct from edit-world IDs, with stable GUID mapping where
  appropriate.
- Runtime-only spawn/despawn policy.
- Variable frame update and fixed-step accumulator skeleton.
- Safe structural-change queues at frame/fixed-step sync points.
- Frame-scoped diagnostics/log attribution.
- Coherent world snapshot boundary for rendering and inspection.

Representative issues:

- Runtime world clone from edit scene.
- Runtime identity mapping.
- Runtime frame step skeleton.
- Fixed-step scheduler skeleton.
- Runtime structural-change sync points.
- Runtime diagnostics/log attribution.

Implementation level: Level 3.

Required tests/checks:

- Level 3 checks from `AGENTS.md`.
- Integration tests for edit-to-runtime cloning and teardown.
- Deterministic frame/fixed-step ordering tests.

Human verification:

- Confirm the runtime model preserves ADR 0019 edit/play boundaries.

## M9: Signal and Event Delivery

Goal: provide deterministic signal/event behavior for scripts, physics, runtime
systems, diagnostics, and MCP inspection.

Key outputs:

- Signal descriptors and stable author-facing names.
- Connection/disconnection lifecycle.
- Deterministic event queues and delivery order.
- Frame-level and fixed-step flush points.
- Cleanup when instances or runtime worlds are destroyed.
- Diagnostics for invalid signal usage.

Representative issues:

- Signal connection handle model.
- Deterministic signal delivery queue.
- Frame/fixed-step signal flush integration.
- Signal cleanup on instance deletion.
- Signal diagnostics.

Implementation level: Level 2.

Required tests/checks:

- Level 2 checks from `AGENTS.md`.
- Determinism tests for delivery order.
- Lifecycle cleanup tests.

Human verification:

- Confirm signal behavior can support Luau-friendly events without exposing
  runtime internals.

## M10: Command and Semantic Change Core

Goal: implement the shared mutation surface used later by editor UI, MCP,
dirty-state tracking, undo/redo, serialization, diagnostics, and tests.

Key outputs:

- Command input/result model.
- Validation-before-mutation contract.
- Structured semantic change records.
- Undo/redo record shape and grouping.
- Dirty-state explanation based on saved snapshots plus change records.
- Command diagnostics for invalid operations.
- Initial scene and project command families.

Representative issues:

- Command result and validation model.
- Semantic change record model.
- Undo/redo stack core.
- Dirty-state explanation core.
- Scene command family.
- Project command family.

Implementation level: Level 3.

Required tests/checks:

- Level 3 checks from `AGENTS.md`.
- Unit tests for validation and failure diagnostics.
- Integration tests for undo/redo and dirty-state explanations.

Human verification:

- Confirm command records express editor and MCP needs before UI handlers are
  built on top.

## M11: Resource Database and Asset Validation

Goal: move from manifest identity to an engine-owned resource database that can
validate references and report asset health before import/rendering work grows.

Key outputs:

- Resource database over committed manifests.
- GUID and `res://` lookup APIs.
- Missing/moved/duplicate asset diagnostics.
- Resource reference validation from scene/property values.
- Import cache state model without requiring full importers yet.
- Asset dependency lookup contract.

Representative issues:

- Resource database scaffold.
- Resource reference validation.
- Missing and duplicate asset diagnostics.
- Import cache state contract.
- Asset dependency lookup contract.

Implementation level: Level 2.

Required tests/checks:

- Level 2 checks from `AGENTS.md`.
- Unit tests for lookup, validation, and diagnostics.
- Golden manifest fixtures once serialization is active.

Human verification:

- Confirm third-party importer types do not leak into public resource APIs.

## M12: Script Runtime Contract Slice

Goal: prove script lifecycle and safe handles at the engine-contract level
before committing to a concrete Luau bridge.

Key outputs:

- Script asset/reference contract.
- Script attachment model.
- Lifecycle scheduling contract for `Ready`, `Update`, `PhysicsUpdate`, and
  `Exit`.
- Safe instance/resource handle access boundaries.
- Script diagnostics and source locations.
- Structural changes from scripts routed through runtime-safe queues.

Representative issues:

- Script attachment contract.
- Script lifecycle dispatch contract.
- Safe script handle API.
- Script diagnostics contract.
- Script structural-change queue integration.

Implementation level: Level 3.

Required tests/checks:

- Level 3 checks from `AGENTS.md`.
- Lifecycle ordering tests using a fake script runtime.
- Diagnostics tests for missing scripts and invalid handles.

Human verification:

- Confirm the contract can support Luau without exposing VM internals.

## M13: Editor Command Surface

Goal: expose project and scene mutation through validated editor commands before
visible editor UI and MCP mutating tools rely on them.

Key outputs:

- Create/delete/rename/reparent instance commands.
- Duplicate instance command.
- Set reflected property command.
- Attach/detach script command.
- Import/change asset setting command shape where resource APIs exist.
- Undo/redo groups and dirty-state explanations.
- Command diagnostics suitable for UI and MCP.

Representative issues:

- Instance mutation commands.
- Property mutation command.
- Script attachment command.
- Asset command scaffold.
- Undo/redo command grouping.
- Dirty-state explanation tests.

Implementation level: Level 3.

Required tests/checks:

- Level 3 checks from `AGENTS.md`.
- Command integration tests against project/scene fixtures.
- Dirty-state and undo/redo tests.

Human verification:

- Confirm editor commands are stable enough to act as automation surfaces.

## M14: MCP Read-Only Automation

Goal: let agents inspect project, scene, runtime, diagnostics, selection, and
resource state through semantic tools.

Key outputs:

- Editor-owned MCP server scaffold.
- `project.open` / `project.create_temp` shape.
- `scene.list_instances`.
- `scene.get_instance`.
- `scene.get_property`.
- `asset.list` / `asset.list_dependencies` shape.
- `diagnostics.list`.
- `editor.get_dirty_state`.
- Test harness hooks.

Representative issues:

- MCP server dependency proposal.
- MCP read-only command schema.
- MCP diagnostics listing.
- MCP scene inspection.
- MCP resource inspection.
- MCP project temp workspace test support.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Schema tests for read-only command responses.
- Fixture-backed tests for scene/resource/diagnostic inspection.

Human verification:

- Confirm read-only MCP reports the same state humans see in project files and
  diagnostics.

## M15: MCP Mutating Automation

Goal: let agents safely edit through the same command path as the editor, with
no second mutation path.

Key outputs:

- `scene.create_instance`.
- `scene.set_property`.
- `scene.reparent_instance`.
- `scene.delete_instance`.
- `editor.undo` / `editor.redo`.
- Mutating MCP commands mapped to editor commands.
- Dirty-state and diagnostics verification.
- Explicit edit/play target-mode handling.

Representative issues:

- MCP create/delete instance commands.
- MCP set property command.
- MCP reparent command.
- MCP undo/redo command.
- MCP command-to-change-record integration tests.
- MCP edit/play ambiguity diagnostics.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Integration tests proving MCP and command APIs produce equivalent change
  records.
- Dirty-state and undo/redo tests through MCP.

Human verification:

- Confirm MCP cannot bypass validation, undo, diagnostics, dirty state, or play
  mode boundaries.

## M16: First Editor Shell

Goal: create the first visible Kinetik Studio loop without pretending the shell
is already a complete editor.

Key outputs:

- Window shell.
- Basic app lifecycle.
- Panel layout for Explorer, Inspector, Diagnostics, and Viewport placeholder.
- Menu/toolbar placeholders for open/save/play actions.
- Manual/screenshot smoke verification.
- No direct project mutation from UI handlers that bypasses commands.

Representative issues:

- Editor shell dependency proposal.
- Window and app shell.
- Panel layout scaffold.
- Diagnostics panel placeholder.
- Viewport placeholder.
- Screenshot smoke.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Manual or automated screenshot smoke.
- No runtime/editor dependency boundary violations.

Human verification:

- Confirm the shell is visually coherent enough to continue editor work.

## M17: Editor Document Session

Goal: give the editor a real document/session model for active project, scene,
selection, diagnostics, dirty state, and mode ownership.

Key outputs:

- Active project/session state.
- Active scene/document state.
- Selection model.
- Dirty-state source wired to command/change records.
- Diagnostics collection and panel data model.
- Open/close project flow.
- Clear edit/play mode ownership.

Representative issues:

- Editor session model.
- Active project and scene document state.
- Editor selection model.
- Editor diagnostics model.
- Editor dirty-state integration.
- Open/close project flow.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Unit/integration tests for selection and dirty-state transitions.
- Manual editor smoke for opening and closing a project.

Human verification:

- Confirm editor state is separate from runtime state and project source state.

## M18: Explorer Scene Editing

Goal: make the Explorer a real scene hierarchy surface backed by editor
commands.

Key outputs:

- Display actual scene hierarchy.
- Select instance.
- Create, delete, rename, duplicate, and reparent through commands.
- Show stable path/GUID information where useful.
- Update dirty state and diagnostics after each command.
- Undo/redo support for Explorer operations.

Representative issues:

- Explorer hierarchy view.
- Explorer selection integration.
- Explorer create/delete/rename commands.
- Explorer reparent/duplicate commands.
- Explorer undo/redo smoke.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Command-level tests plus editor smoke.
- Screenshot/manual verification for common hierarchy edits.

Human verification:

- Confirm hierarchy edits feel understandable and do not mutate unrelated state.

## M19: Inspector Property Editing

Goal: make the Inspector consume reflection metadata and edit properties through
the same validation path as commands and MCP.

Key outputs:

- Basic typed field rendering for strings, numbers, bools, vectors, colors, and
  resource/instance references as available.
- Property edits routed through `SetProperty`.
- Read-only and validation diagnostics displayed.
- Undo/redo support for property edits.
- Inspector and MCP property behavior aligned.

Representative issues:

- Inspector descriptor rendering.
- Inspector basic typed fields.
- Inspector set-property command integration.
- Inspector validation diagnostics.
- Inspector undo/redo smoke.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Property validation tests.
- Screenshot/manual verification for common property edits.

Human verification:

- Confirm the Inspector is clear, typed, and does not invent editor-only
  property rules.

## M20: Project Save/Reload From Editor

Goal: prove the editor can persist and reload project state deterministically.

Key outputs:

- Save active project/scene/manifest state through approved serialization
  boundaries.
- Reload saved project into equivalent source state.
- Dirty state clears only when saved snapshots match.
- Preserve GUIDs, hierarchy, properties, and manifest references.
- Golden fixtures for editor save/reload.

Representative issues:

- Editor save command.
- Editor reload command.
- Save/reload dirty-state tests.
- Editor project golden fixtures.
- Save/reload diagnostics.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Golden fixture tests.
- Editor smoke for save, reload, and dirty-state explanation.

Human verification:

- Confirm saved files are deterministic and reviewable in Git.

## M21: MCP Editor Parity Slice

Goal: prove UI actions and agent actions observe and mutate the same editor
state through the same command/change path.

Key outputs:

- MCP reports active project, scene, selection, diagnostics, and dirty state.
- MCP mutating commands produce the same change records as UI commands.
- UI updates after MCP changes.
- MCP selection/focus commands where useful.
- Parity tests for representative Explorer and Inspector workflows.

Representative issues:

- MCP active editor state.
- MCP selection and focus commands.
- MCP/UI command parity tests.
- MCP diagnostics parity.
- MCP dirty-state parity.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- MCP-driven editor integration tests.
- Manual editor smoke for MCP-triggered changes.

Human verification:

- Confirm agent automation does not create state the editor cannot explain.

## M22: Play Mode Control Slice

Goal: add editor play controls that exercise runtime sandboxing without leaking
runtime changes into saved edit state.

Key outputs:

- Play, stop, and step controls.
- Runtime world created from current edit scene.
- Runtime IDs distinct from edit IDs.
- Stop destroys play world.
- Runtime diagnostics visible in editor/MCP.
- Ambiguous edit/play commands fail with diagnostics.

Representative issues:

- Editor play/start/stop controls.
- Runtime sandbox integration.
- Runtime diagnostics panel integration.
- MCP play.start/play.step/play.stop.
- Edit/play mutation boundary tests.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Lifecycle tests for play, step, stop, and teardown.
- MCP/headless play-mode smoke.

Human verification:

- Confirm play mode never silently persists runtime-only mutations.

## M23: Viewport Interaction Scaffold

Goal: establish viewport interaction foundations before rendering and 3D
authoring depend on them.

Key outputs:

- Viewport panel owns camera/navigation state.
- Focus selected instance.
- Basic selection highlight or overlay.
- Placeholder picking/focus contract.
- Screenshot smoke for viewport layout and selection context.
- Path toward runtime renderer usage without a separate long-term renderer.

Representative issues:

- Viewport camera/navigation state.
- Viewport focus selected.
- Viewport selection overlay.
- Viewport picking contract.
- Viewport screenshot smoke.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Screenshot/manual verification.
- No divergence from planned runtime renderer path.

Human verification:

- Confirm viewport interactions are understandable enough to support 3D
  authoring work.

## M24: First 3D Scene Authoring Slice

Goal: prove a complete 3D scene-data authoring loop before requiring final
rendering quality.

Key outputs:

- Create/scaffold project.
- Load scene.
- Add 3D scene instances as data.
- Set transform and material-facing reflected properties.
- Save and reload.
- Enter play mode.
- Run deterministic frame steps.
- Inspect diagnostics.
- Verify through MCP or headless automation.

Representative issues:

- Hello scene upgrade to 3D scene data.
- End-to-end project scaffold/load/save test.
- First play-mode smoke.
- MCP-driven 3D authoring smoke.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Golden project/scene fixture.
- MCP or headless smoke for create, save, reload, play, step, and diagnostics.

Human verification:

- Confirm the editor can author and preserve the 3D scene data coherently.

## M25: First Rendered Primitive Scene

Goal: make runtime and editor rendering capable enough to display a primitive
scene while preserving the long-term renderer direction.

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

## M26: Template Project Contract

Goal: define how first-party templates live in the repository before content
templates become executable acceptance targets.

Key outputs:

- Decision on `templates/` versus `examples/` for first-party template projects.
- Template README and verification-note format.
- Screenshot/golden reference policy.
- Template fixture determinism rules.
- Known-limitation section format.
- CI/headless/human verification expectations.

Representative issues:

- Template directory contract.
- Template verification note format.
- Template screenshot/golden policy.
- Template CI fixture policy.

Implementation level: Level 0.

Required tests/checks:

- Relevant docs/ADR consistency checks.

Human verification:

- Approve where template projects live before content work starts.

## M27: Primitive Showcase Template

Goal: ship the first Kinetik template proving basic 3D authoring, hierarchy,
transforms, materials, viewport inspection, and play-mode smoke work together.

Key outputs:

- `templates/primitive_showcase` project, or the approved equivalent location.
- Curated scene containing all supported built-in primitive meshes.
- Clear hierarchy, names, transforms, and material assignments.
- Camera and lighting setup suitable for immediate inspection.
- Save/reload fixture or golden output for the authored scene.
- Verification notes describing what the template proves and known limitations.

Representative issues:

- Primitive showcase scene authoring fixture.
- Primitive transform/material editing smoke.
- Primitive showcase save/reload golden.
- Primitive showcase editor screenshot verification.

Implementation level: Level 5.

Required tests/checks:

- Level 5 checks from `AGENTS.md`.
- Golden fixture for the template scene.
- MCP or headless smoke for open, inspect hierarchy, enter play mode, step, and
  stop.

Human verification:

- Confirm the scene is visually legible, primitives are recognizable, editor
  selection/inspection is usable, and play mode does not persist temporary
  runtime mutations.

## M28: Asset Import and Material Foundation

Goal: build the import/material foundation required before a PBR demo claims to
exercise real asset workflows.

Key outputs:

- Texture import/reimport/cache smoke.
- Mesh import smoke for glTF/GLB once approved.
- Material asset/reference validation.
- Import settings validation and diagnostics.
- Missing/invalid resource diagnostics.
- Imported artifact records in disposable cache state.

Representative issues:

- Asset import dependency installation after approval.
- Texture import smoke.
- glTF/GLB mesh import smoke.
- Material asset reference validation.
- Import cache artifact records.
- Import diagnostics.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Import fixture tests.
- Golden manifest/cache metadata tests where stable.
- Diagnostics tests for missing or invalid resources.

Human verification:

- Confirm imported assets remain deterministic and source assets/manifests stay
  reviewable.

## M29: PBR Material Demo Scene

Goal: demonstrate Kinetik's practical PBR direction with a compact scene that
exercises material, lighting, texture, import, and render-diagnostic workflows.

Key outputs:

- `templates/pbr_material_demo` project, or the approved equivalent location.
- Metallic/roughness material range display.
- Normal map and emissive material examples when supported.
- Directional and local light examples matching the current renderer stage.
- Imported mesh and texture path through the asset manifest.
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
- Integration/golden tests for material and texture references.
- Screenshot smoke for expected material and lighting visibility.

Human verification:

- Confirm the scene communicates the current material model honestly: no
  overpromising unsupported shadows, IBL, graph features, or post-processing.

## M30: Input, Physics, and Interaction Foundation

Goal: establish the gameplay foundations needed before the FPS prototype becomes
a template rather than a hardcoded demo.

Key outputs:

- Input mapping and event model.
- Mouse capture/look policy.
- Static collision against primitive level geometry.
- Character body/controller slice.
- Raycast or proximity interaction primitive.
- Physics diagnostics and headless deterministic checks where practical.

Representative issues:

- Input dependency and API proposal.
- Input mapping runtime slice.
- Mouse capture/look contract.
- Static collision smoke.
- Character controller slice.
- Raycast/proximity interaction slice.
- Physics diagnostics.

Implementation level: Level 4.

Required tests/checks:

- Level 4 checks from `AGENTS.md`.
- Unit tests for deterministic controller math where practical.
- Integration/headless tests for collision and interaction state.

Human verification:

- Confirm input and character movement policy is acceptable before content
  templates depend on it.

## M31: Basic FPS Prototype

Goal: prove Kinetik can author and run a minimal playable 3D game loop, not just
render static scenes.

Key outputs:

- `templates/basic_fps` project, or the approved equivalent location.
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
