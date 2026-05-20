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
- `assets.knmanifest` in-memory model.
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

Goal: review dependency choices before implementation crates add external
libraries or shape public APIs around them.

Key outputs:

- Surveys/proposals for core utilities, runtime/app, renderer, editor/window/UI,
  physics, asset import, and Luau.
- Explicit crate ownership, license/safety review, transitive-risk notes, unsafe
  exposure, platform support, build impact, and public API boundary guidance.
- Serialization dependency proposal completed or updated for `serde`, TOML, and
  RON.

Representative issues: core dependency survey; renderer dependency proposal;
editor/window/UI dependency proposal; physics, asset import, and Luau dependency
proposals.

Implementation level: Level 0.
Required tests/checks: docs/ADR consistency checks.
Human verification: approve dependencies separately before installation.

## M5: Internal API Contract Specs

Goal: define internal contracts before agents implement runtime, editor, MCP, or
template behavior that depends on them.

Key outputs:

- Specs for `Project`, `Scene`, `Reflection`, `DiagnosticsStore`, `Command` /
  `ChangeRecord`, `RuntimeWorld`, `FrameScheduler`, `ResourceDatabase`,
  `EditorSession`, and MCP internal command surfaces.
- Each spec names owning crates, dependency boundaries, serialized-format
  impact, diagnostics behavior, and public API constraints.

Representative issues: project/editor session spec; scene/reflection spec;
diagnostics spec; command/change-record spec; runtime/frame spec; resource
database spec; MCP command surface spec.

Implementation level: Level 0.
Required tests/checks: docs/ADR consistency checks.
Human verification: confirm specs are sufficient to prevent invented APIs.

## M6: Project Model and Diagnostics Store

Goal: create the engine-owned project/document health layer shared by
serialization, editor, MCP, and tests.

Key outputs:

- Project identity/settings model for `Kinetik.toml`.
- Project layout validation wired to structured diagnostics.
- Active scene/document references without editor-only state.
- Diagnostics store for current health, filtering, blocking scopes, and
  repairability.

Representative issues: project model scaffold; project diagnostics store;
layout validation diagnostics; project fixture helpers.

Implementation level: Level 2.
Required tests/checks: Level 2 checks, layout/diagnostic unit tests, golden
fixtures once serialization is active.
Human verification: confirm no dependency on editor crates or UI state.

## M7: Engine Class and Spatial Model

Goal: deepen the instance model enough for real 3D authoring without coupling it
to rendering, physics, or editor UI.

Key outputs:

- Built-in class descriptors beyond root services.
- Class capability metadata or inheritance/composition policy.
- Local transform property contract and world-transform derivation.
- Deterministic traversal, transform dirty/update behavior, and bounds contract.

Representative issues: built-in 3D class descriptor set; transform contract;
world transform derivation; spatial bounds contract.

Implementation level: Level 2.
Required tests/checks: Level 2 checks, traversal/transform determinism tests,
invalid class/property diagnostics.
Human verification: confirm class names and property paths match reflection and
Luau direction.

## M8: Runtime World and Frame Kernel

Goal: establish runtime world identity and deterministic frame stepping before
script, physics, render, play mode, or MCP runtime inspection depend on it.

Key outputs:

- Runtime world derived from edit scene/document state.
- Runtime IDs distinct from edit IDs, with GUID mapping where appropriate.
- Runtime-only spawn/despawn policy.
- Variable update, fixed-step accumulator, safe structural-change sync points,
  and coherent snapshot boundary.

Representative issues: runtime world clone; runtime identity mapping; frame step
skeleton; fixed-step scheduler; runtime diagnostics/log attribution.

Implementation level: Level 3.
Required tests/checks: Level 3 checks, edit-to-runtime cloning tests,
deterministic frame/fixed-step ordering tests.
Human verification: confirm ADR 0019 edit/play boundaries are preserved.

## M9: Signal and Event Delivery

Goal: provide deterministic signal/event behavior for scripts, physics, runtime
systems, diagnostics, and MCP inspection.

Key outputs:

- Signal descriptors and stable author-facing names.
- Connection/disconnection lifecycle.
- Deterministic event queues and delivery order.
- Frame-level/fixed-step flush points and cleanup on instance/world teardown.

Representative issues: signal connection handles; deterministic delivery queue;
flush integration; cleanup and diagnostics; signal bus internal API contract.

Implementation level: Level 2.
Required tests/checks: Level 2 checks, delivery-order determinism tests,
lifecycle cleanup tests.
Human verification: confirm Luau-friendly events can be supported safely.

## M10: Command and Semantic Change Core

Goal: implement the shared mutation surface for editor UI, MCP, dirty-state
tracking, undo/redo, serialization, diagnostics, and tests.

Key outputs:

- Command input/result model with validation before mutation.
- Structured semantic change records.
- Undo/redo record shape and grouping.
- Dirty-state explanation from saved snapshots and change records.
- Initial scene and project command families.

Representative issues: command result model; semantic change records; undo/redo
core; dirty-state explanation; scene/project command families.

Implementation level: Level 3.
Required tests/checks: Level 3 checks, validation diagnostics, undo/redo and
dirty-state integration tests.
Human verification: confirm command records express editor and MCP needs before
UI handlers are built.

## M11: Resource Database and Asset Validation

Goal: move from manifest identity to an engine-owned resource database that can
validate references and report asset health.

Key outputs:

- Resource database over committed manifests.
- GUID and `res://` lookup APIs.
- Missing/moved/duplicate asset diagnostics.
- Resource reference validation from scene/property values.
- Import cache state model and asset dependency lookup contract.

Representative issues: resource database scaffold; reference validation;
missing/duplicate diagnostics; import cache state; dependency lookup.

Implementation level: Level 2.
Required tests/checks: Level 2 checks, lookup/validation/diagnostics tests,
golden manifest fixtures once serialization is active.
Human verification: confirm importer types do not leak into public resource APIs.

## M12: Script Runtime Contract Slice

Goal: prove script lifecycle and safe handles at the engine-contract level
before committing to a concrete Luau bridge.

Key outputs:

- Script asset/reference and attachment contracts.
- Lifecycle scheduling for `Ready`, `Update`, `PhysicsUpdate`, and `Exit`.
- Safe instance/resource handle boundaries.
- Script diagnostics and queued structural changes from scripts.

Representative issues: script attachment contract; lifecycle dispatch contract;
safe script handle API; script diagnostics; structural-change queue integration.

Implementation level: Level 3.
Required tests/checks: Level 3 checks, fake-runtime lifecycle ordering tests,
missing-script and invalid-handle diagnostics.
Human verification: confirm the contract can support Luau without VM internals
leaking.

## M13: Editor Command Surface

Goal: expose project and scene mutation through validated editor commands before
visible editor UI and MCP mutating tools rely on them.

Key outputs:

- Create/delete/rename/reparent/duplicate instance commands.
- Set reflected property command.
- Attach/detach script command.
- Asset command shape where resource APIs exist.
- Undo/redo groups, dirty-state explanations, and command diagnostics.

Representative issues: instance mutation commands; property command; script
attachment command; asset command scaffold; undo/redo grouping.

Implementation level: Level 3.
Required tests/checks: Level 3 checks, command integration tests, dirty-state
and undo/redo tests.
Human verification: confirm commands are stable automation surfaces.

## M14: MCP Read-Only Automation

Goal: let agents inspect project, scene, runtime, diagnostics, selection, and
resource state through semantic tools.

Key outputs:

- Editor-owned MCP server scaffold.
- Read-only project, scene, property, resource, diagnostics, and dirty-state
  commands.
- Fixture-backed test harness hooks.

Representative issues: MCP server dependency proposal; read-only command
schema; diagnostics listing; scene/resource inspection; temp workspace support.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, schema tests, fixture-backed inspection
tests.
Human verification: confirm MCP reports the same state humans see.

## M15: MCP Mutating Automation

Goal: let agents safely edit through the same command path as the editor, with
no second mutation path.

Key outputs:

- Mutating scene/property/undo/redo MCP commands mapped to editor commands.
- Dirty-state and diagnostics verification.
- Explicit edit/play target-mode handling.

Representative issues: MCP create/delete/reparent/set-property commands; MCP
undo/redo; command-to-change-record tests; edit/play ambiguity diagnostics.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, MCP/command parity tests, dirty-state and
undo/redo tests through MCP.
Human verification: confirm MCP cannot bypass validation or play boundaries.

## M16: First Editor Shell

Goal: create the first visible Kinetik Studio loop without pretending the shell
is already a complete editor.

Key outputs:

- Window shell and basic app lifecycle.
- Panel layout for Explorer, Inspector, Diagnostics, and Viewport placeholder.
- Menu/toolbar placeholders for open/save/play actions.
- Manual/screenshot smoke verification.

Representative issues: editor shell dependency proposal; window/app shell;
panel layout scaffold; diagnostics panel placeholder; viewport placeholder.

Implementation level: Level 5.
Required tests/checks: Level 5 checks, screenshot smoke, boundary checks.
Human verification: confirm the shell is visually coherent enough to continue.

## M17: Editor Document Session

Goal: give the editor real session state for active project, scene, selection,
diagnostics, dirty state, and mode ownership.

Key outputs:

- Active project and scene document state.
- Selection model.
- Dirty-state source wired to command/change records.
- Diagnostics collection and panel data model.
- Open/close project flow and edit/play mode ownership.

Representative issues: editor session model; active project/scene state;
selection model; diagnostics model; dirty-state integration.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, selection/dirty-state tests, open/close
project smoke.
Human verification: confirm editor state is separate from runtime/source state.

## M18: Explorer Scene Editing

Goal: make the Explorer a real hierarchy surface backed by editor commands.

Key outputs:

- Display actual scene hierarchy.
- Select, create, delete, rename, duplicate, and reparent through commands.
- Show stable path/GUID information where useful.
- Update dirty state and diagnostics, with undo/redo support.

Representative issues: hierarchy view; selection integration; create/delete/
rename; reparent/duplicate; undo/redo smoke.

Implementation level: Level 5.
Required tests/checks: Level 5 checks, command tests, screenshot/manual
verification.
Human verification: confirm hierarchy edits are understandable and scoped.

## M19: Inspector Property Editing

Goal: make the Inspector consume reflection metadata and edit properties through
the same validation path as commands and MCP.

Key outputs:

- Basic typed fields for common reflected value types.
- Property edits routed through `SetProperty`.
- Read-only and validation diagnostics displayed.
- Undo/redo support and MCP property behavior parity.

Representative issues: descriptor rendering; typed fields; set-property
integration; validation diagnostics; undo/redo smoke.

Implementation level: Level 5.
Required tests/checks: Level 5 checks, property validation tests, screenshot/
manual verification.
Human verification: confirm the Inspector does not invent editor-only rules.

## M20: Project Save/Reload From Editor

Goal: prove the editor can persist and reload project state deterministically.

Key outputs:

- Save and reload active project/scene/manifest state through approved
  serialization boundaries.
- Dirty state clears only when saved snapshots match.
- GUIDs, hierarchy, properties, and manifest references are preserved.
- Golden fixtures for editor save/reload.

Representative issues: editor save command; reload command; dirty-state tests;
golden fixtures; save/reload diagnostics.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, golden fixture tests, editor save/reload
smoke.
Human verification: confirm saved files are deterministic and Git-reviewable.

## M21: MCP Editor Parity Slice

Goal: prove UI actions and agent actions observe and mutate the same editor
state through the same command/change path.

Key outputs:

- MCP reports active project, scene, selection, diagnostics, and dirty state.
- MCP mutating commands produce the same change records as UI commands.
- UI updates after MCP changes.

Representative issues: MCP active editor state; selection/focus commands;
MCP/UI parity tests; diagnostics and dirty-state parity.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, MCP-driven editor integration tests.
Human verification: confirm agent automation creates explainable editor state.

## M22: Play Mode Control Slice

Goal: add editor play controls that exercise runtime sandboxing without leaking
runtime changes into saved edit state.

Key outputs:

- Play, stop, and step controls.
- Runtime world created from current edit scene.
- Runtime IDs distinct from edit IDs; stop destroys play world.
- Runtime diagnostics visible in editor/MCP.
- Ambiguous edit/play commands fail with diagnostics.

Representative issues: play/start/stop controls; runtime sandbox integration;
runtime diagnostics panel integration; MCP play commands; boundary tests.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, play lifecycle tests, MCP/headless smoke.
Human verification: confirm play mode does not persist runtime-only mutations.

## M23: Viewport Interaction Scaffold

Goal: establish viewport interaction foundations before rendering and 3D
authoring depend on them.

Key outputs:

- Viewport camera/navigation state.
- Focus selected instance.
- Basic selection highlight or overlay.
- Placeholder picking/focus contract.
- Screenshot smoke and path toward runtime renderer usage.

Representative issues: viewport camera/navigation; focus selected; selection
overlay; picking contract; screenshot smoke.

Implementation level: Level 5.
Required tests/checks: Level 5 checks, screenshot/manual verification.
Human verification: confirm interactions can support 3D authoring.

## M24: First 3D Scene Authoring Slice

Goal: prove a complete 3D scene-data authoring loop before requiring final
rendering quality.

Key outputs:

- Create/scaffold project, load scene, add 3D scene instances as data, set
  transform/material-facing properties, save/reload, enter play mode, step, and
  inspect diagnostics.
- MCP or headless verification for the full loop.

Representative issues: 3D hello scene data; scaffold/load/save test; first
play-mode smoke; MCP-driven 3D authoring smoke.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, golden project/scene fixture,
MCP/headless smoke.
Human verification: confirm the editor preserves 3D scene data coherently.

## M25: First Rendered Primitive Scene

Goal: render a primitive scene through the runtime/editor path while preserving
the long-term renderer direction.

Key outputs:

- Camera and light instances.
- Built-in primitive mesh resources.
- Mesh/material extraction boundary.
- PBR-compatible `StandardMaterial` scaffold and safe fallback materials.
- Render diagnostics for missing cameras, meshes, materials, shaders, textures,
  and lights.

Representative issues: renderer dependency confirmation; primitive meshes;
camera/light extraction; material scaffold; fallback diagnostics; viewport render.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, primitive/material unit tests, extraction
diagnostics, screenshot smoke.
Human verification: confirm stable framing and useful render diagnostics.

## M26: Template Project Contract

Goal: define how first-party templates live in the repository before template
content becomes executable acceptance targets.

Key outputs:

- Decision on `templates/` versus `examples/`.
- Template README/verification-note format.
- Screenshot/golden policy, fixture determinism rules, and CI/headless/human
  verification expectations.

Representative issues: template directory contract; verification note format;
screenshot/golden policy; CI fixture policy.

Implementation level: Level 0.
Required tests/checks: docs/ADR consistency checks.
Human verification: approve template location before content work starts.

## M27: Primitive Showcase Template

Goal: ship the first Kinetik template proving basic 3D authoring, hierarchy,
transforms, materials, viewport inspection, and play-mode smoke work together.

Key outputs:

- Approved-location primitive showcase project.
- Curated primitive scene with clear hierarchy, transforms, materials, camera,
  lighting, save/reload fixture, screenshots, and verification notes.

Representative issues: primitive scene fixture; transform/material editing
smoke; save/reload golden; editor screenshot verification.

Implementation level: Level 5.
Required tests/checks: Level 5 checks, golden fixture, MCP/headless smoke.
Human verification: confirm visual legibility and no play-state persistence.

## M28: Asset Import and Material Foundation

Goal: build import/material foundation before a PBR demo claims real asset
workflow coverage.

Key outputs:

- Texture import/reimport/cache smoke.
- glTF/GLB mesh import smoke once approved.
- Material asset/reference validation.
- Import settings/cache artifact records and diagnostics.

Representative issues: import dependency installation after approval; texture
import; glTF/GLB import; material reference validation; import diagnostics.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, import fixtures, manifest/cache metadata
tests, diagnostics tests.
Human verification: confirm assets remain deterministic and reviewable.

## M29: PBR Material Demo Scene

Goal: demonstrate Kinetik's practical PBR direction with a compact material,
lighting, texture, import, and render-diagnostic scene.

Key outputs:

- Approved-location PBR material demo project.
- Metallic/roughness range, normal/emissive examples when supported, directional
  and local lighting, imported mesh/texture path, diagnostics, and screenshots.

Representative issues: PBR material range fixture; texture/material import
smoke; light setup; imported mesh render smoke; screenshot verification.

Implementation level: Level 5.
Required tests/checks: Level 5 checks, material validation, golden/reference
tests, screenshot smoke.
Human verification: confirm the scene does not overpromise unsupported renderer
features.

## M30: Input, Physics, and Interaction Foundation

Goal: establish gameplay foundations before the FPS prototype becomes a
template instead of a hardcoded demo.

Key outputs:

- Input mapping and event model.
- Mouse capture/look policy.
- Static collision against primitive level geometry.
- Character controller slice.
- Raycast or proximity interaction primitive.

Representative issues: input dependency/API proposal; input mapping; mouse
capture/look; static collision; character controller; interaction primitive.

Implementation level: Level 4.
Required tests/checks: Level 4 checks, controller math tests where practical,
collision/interaction integration tests.
Human verification: confirm movement/input policy before templates depend on it.

## M31: Basic FPS Prototype

Goal: prove Kinetik can author and run a minimal playable 3D game loop, not just
render static scenes.

Key outputs:

- Approved-location basic FPS project.
- First-person camera/controller, static collision, simple interaction, minimal
  start/move/look/interact/complete/restart loop, diagnostics, and playtest
  checklist.

Representative issues: FPS task contract/control scheme; controller slice;
collision smoke; interaction objective; play-mode diagnostics and restart smoke.

Implementation level: Level 5.
Required tests/checks: Level 5 checks, controller tests where practical,
headless lifecycle/objective tests, MCP play smoke.
Human verification: confirm movement, mouse look, interaction, and restart feel
acceptable for a prototype.

## M32: Post-Template Feature Roadmap

Goal: keep a record of major engine/editor capabilities that remain planned
after the first 3D template set, without pulling them into the primitive, PBR,
or basic FPS acceptance scope.

Key outputs:

- Deferred feature backlog grouped by subsystem.
- Dependency/API/spec gates identified before each feature family starts.
- Clear distinction between first-template requirements and later engine
  maturity work.

Deferred feature areas:

- Audio: buses, playback, spatial audio, editor preview, and mixing workflows.
- Animation: clips, skeletal import, animation state, retargeting direction, and
  editor preview.
- Runtime UI: UI scene instances, layout, input focus, styling, and menu/HUD
  workflows.
- Prefabs and packages: clone-ready templates, override records, package
  dependencies, and broken-override diagnostics.
- Build/export/bundles: `.knbundle` build/load/verify, platform export,
  signing/hash verification, and runtime content mounting.
- Terrain and world environment: terrain chunks, brushes, sky/atmosphere,
  time-of-day, weather direction, and large-world constraints.
- Advanced rendering: shadows, HDR/tone mapping, environment lighting, IBL,
  render graph, material graph, shader graph authoring, and Forward+ direction.
- Asset pipeline expansion: richer model/material import, texture compression,
  reimport policies, repair tools, and asset dependency visualization.
- Editor polish: docking/layout persistence, keyboard shortcuts, accessibility,
  profiling panels, visual regression harness, and richer viewport gizmos.
- Scripting maturity: Luau type generation, debugger hooks, hot reload policy,
  API documentation, and sandbox permissions.
- Physics maturity: joints, triggers/areas, character-controller refinement,
  collision layers UI, events, debug drawing, and editor quick fixes.
- Networking/multiplayer: explicitly post-first-template until local runtime,
  scripting, diagnostics, and editor workflows are proven.
- 2D support: explicitly post-first-template while Kinetik remains optimized for
  the first 3D milestones.

Representative issues:

- Future audio foundation roadmap.
- Future animation foundation roadmap.
- Future runtime UI roadmap.
- Future prefab/package implementation roadmap.
- Future build/export/bundle roadmap.
- Future terrain/world roadmap.
- Future advanced renderer roadmap.
- Future editor polish roadmap.

Implementation level: Level 0.
Required tests/checks: docs/ADR consistency checks.
Human verification: confirm each deferred feature family gets dependency review
and internal API specs before implementation issues are opened.
