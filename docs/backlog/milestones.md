# Kinetik Studio Roadmap

This is the single ordered roadmap for `kinetik-engine`.

The current product target is not a demo window, a headless fixture suite, or a
static preview. The target is a usable, production-minded game editor that can
open a project, show the engine, let a user edit the scene, run it, save it, and
explain what went wrong when something fails.

## Product Target

Kinetik Studio is usable when a developer can:

- Create a project from a first-party template or open an existing project.
- See the active scene in the editor window, not just in a test harness.
- Select scene objects from the viewport or hierarchy.
- Inspect and edit common object properties through the UI.
- Add, delete, duplicate, and reparent objects with undo and redo.
- Save, close, reopen, and get the same project state back.
- Press play, run the scene through the engine, stop play, and return to edit
  mode without corrupting project state.
- See diagnostics for invalid assets, scripts, project files, rendering setup,
  and runtime failures.
- Import basic assets and understand missing or stale asset state.
- Build or run an exported artifact through a minimal, documented path.

The software-rendered template cards shipped after M32 are an emergency bridge:
they prove the window paints and first-party templates are visible. They are not
the editor goal. Any future milestone that says "editor UI" must be verifiable
inside the app window, not only through headless tests.

## Shipped Foundation

The detailed M1-M32 history has been truncated because it is no longer the live
plan. The following foundation is already shipped and should be preserved:

- Core engine crates, project model, scene graph, transforms, serialization,
  identifiers, diagnostics, and typed command surfaces.
- Runtime frame loop, schedule phases, script lifecycle contracts, edit/play
  boundary model, signal and event plumbing, and project validation.
- Reflection/property metadata, inspector data model, explorer/session state,
  undoable editor commands, MCP/editor protocol slices, and headless editor
  smoke coverage.
- Render extraction contracts, primitives, materials, renderer diagnostics,
  camera/light records, template manifests, and first-party primitive, PBR, and
  basic FPS templates.
- Kinetik Studio window creation, visible software-rendered template cards, and
  an idle redraw fix so the window does not burn the event loop while idle.

Current limits that must not be hidden by planning language:

- Studio does not yet have a real project/template launcher workflow.
- The explorer and inspector panels are not wired to interactive user actions.
- The viewport is not a full active scene editor viewport.
- The GPU renderer is not embedded as the primary Studio viewport.
- There is no viewport selection, camera navigation, transform gizmo, or overlay
  feedback in the app window.
- Templates are visible, but they are not yet open-edit-play-save workflows.
- Asset import, project save/reload UX, script authoring UX, and export UX are
  still missing.

## Roadmap Rules

- Work on exactly one issue or backlog slice per PR.
- Keep slices reviewable and production-quality. Do not hide broad subsystem
  rewrites inside editor milestones.
- Required baseline checks remain:
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- Run `cargo doc --workspace --no-deps` when public APIs change.
- Run fixture, golden, integration, headless runtime/editor, MCP, screenshot, or
  manual UI checks required by the touched subsystem.
- UI-facing milestones require app-window verification. Prefer screenshot or
  deterministic smoke evidence in addition to automated model tests.
- Do not add or upgrade dependencies without approval.
- Do not introduce, wrap, expand, or lint-suppress unsafe Rust without approval.
- Do not change public APIs, serialized formats, generated contracts, migrations,
  editor/runtime boundaries, or architecture direction unless an accepted ADR or
  internal spec explicitly covers the change.
- Keep accepted Lua/Luau support intact. Future `kinetik-lang` `.kn` support is
  additive and needs scoped issues/specs before changing scripting direction.
- When a code-health problem is outside the active slice, file a scoped follow-up
  issue with file paths, risk, and acceptance criteria instead of broadening the
  PR.

## Editor Milestones

### E1: Real Project and Template Launcher

Goal: Studio starts as usable software, not a passive canvas.

Required behavior:

- Show a Studio home view with first-party template entries and an open-project
  entry point.
- Let the user create a project from `primitive-showcase`, `pbr-material-demo`,
  and `basic-fps` templates.
- Let the user open an existing project path through the UI path supported by the
  platform shell.
- Surface project/template load errors in the window and logs.
- Transition from launcher state into an active editor session.

Acceptance:

- Template creation and project opening are covered by automated tests around
  the editor/session model.
- A manual or screenshot smoke proves the launcher is visible and at least one
  template transitions into an active editor view.
- No busy idle redraw loop returns.

### E2: Explorer and Inspector Wiring

Goal: the side panels represent the actual project state and can drive editing.

Required behavior:

- Explorer lists the active scene hierarchy from the loaded project.
- Selection in the explorer updates editor selection state.
- Inspector shows common selected-object properties: name, transform, enabled
  state, material/render records when present, and script records when present.
- Editing supported inspector fields dispatches existing undoable commands.
- Invalid edits produce visible diagnostics instead of silent failures.

Acceptance:

- Model tests prove selection and inspector edits flow through commands.
- Manual or screenshot smoke proves hierarchy and inspector panels are populated
  from a loaded template.

### E3: Active Software Viewport Baseline

Goal: replace static template cards with one active scene viewport that responds
to the editor session.

Required behavior:

- Studio shows one active scene viewport after a project/template is loaded.
- The viewport draws the loaded scene with the existing software presentation
  path while the GPU viewport is prepared.
- Selection changes are visible in the viewport.
- Resize and scale-factor changes keep the viewport correctly framed.
- The window repaints on actual state changes without spinning while idle.

Acceptance:

- Automated smoke covers viewport model output for all first-party templates.
- Manual or screenshot smoke proves the active viewport is visible in the app
  window after opening a template.

### E4: Editing Actions, Undo, and Redo

Goal: users can change a scene without editing files by hand.

Required behavior:

- Add, delete, duplicate, rename, and reparent scene objects from the UI.
- Edit transform values from the inspector.
- Provide toolbar/menu/keyboard entry points for undo and redo.
- Show dirty state after edits and clear it after save.
- Keep command ownership explicit and testable.

Acceptance:

- Command tests cover each edit action and undo/redo behavior.
- Manual smoke proves a user can edit a template scene, undo, redo, and see the
  UI update.

### E5: Save, Reload, and Recent Projects

Goal: project state survives normal editor use.

Required behavior:

- Save the active project through the UI.
- Reload a saved project and preserve edited hierarchy, transforms, material
  records, and script records supported by the current schema.
- Warn before destructive close/open actions when the project is dirty.
- Track recent projects using a deterministic local settings file.
- Report save/load errors in diagnostics.

Acceptance:

- Round-trip tests cover edited project save/reload.
- Manual smoke proves edit, save, close/reopen, and inspect works from Studio.

### E6: Play Mode in the UI

Goal: the editor can run the game and return to editing.

Required behavior:

- Toolbar exposes play, stop, and pause/step if supported by the runtime model.
- Play mode runs the active scene through runtime entry points.
- Edit mode state is restored after stop.
- Runtime diagnostics and script failures are visible in Studio.
- Template play requirements are explicit in template manifests or docs.

Acceptance:

- Runtime/editor tests prove edit-to-play and play-to-edit state transitions.
- Manual smoke proves at least one first-party template can be played and stopped
  from the app window.

### E7: First True GPU Viewport

Goal: the engine renderer appears inside Studio as the primary viewport.

Required behavior:

- Embed a renderer-backed viewport surface in the editor window.
- Render the active scene camera, primitives, materials, and lights through the
  engine rendering path.
- Handle resize, scale factor, device loss, and renderer initialization failures
  with diagnostics.
- Provide a software or diagnostics fallback only when GPU initialization fails.
- Keep renderer/editor ownership boundaries consistent with architecture docs and
  ADRs.

Acceptance:

- App-window smoke proves the GPU viewport is visible for at least one template.
- Renderer diagnostics are covered by automated tests where practical.
- `cargo doc --workspace --no-deps` runs if public renderer/editor APIs change.

### E8: Viewport Navigation, Selection, and Gizmos

Goal: the viewport becomes an editing surface, not just a render target.

Required behavior:

- Navigate the editor camera with documented mouse/keyboard controls.
- Select objects in the viewport.
- Highlight selected objects.
- Show transform gizmos for translate at minimum; rotate/scale can be separate
  follow-ups if scoped.
- Frame/focus selected objects.
- Keep picking and gizmo math deterministic and testable outside the window loop.

Acceptance:

- Unit tests cover picking/gizmo math.
- Manual smoke proves viewport select, focus, and transform edit works in Studio.

### E9: Asset Browser and Import Workflow

Goal: users can bring basic content into a project and inspect asset health.

Required behavior:

- Show project assets in a browser panel.
- Import or register supported baseline assets: textures, materials, and simple
  mesh/scene assets covered by existing accepted architecture.
- Report missing, stale, invalid, and unsupported assets.
- Let users assign supported assets to scene objects through the inspector.
- Keep import metadata deterministic and source-control friendly.

Acceptance:

- Fixture tests cover asset registration and diagnostics.
- Manual smoke proves an asset can be imported or assigned through Studio.

### E10: Template Projects Work End-to-End

Goal: first-party templates are usable projects in the UI.

Required behavior:

- `primitive-showcase`, `pbr-material-demo`, and `basic-fps` open from the
  launcher, show in the viewport, populate explorer/inspector, support at least
  one meaningful edit, save/reload, and play when the template has runtime
  behavior.
- Template manifests describe editor-facing expectations.
- Failures include actionable diagnostics.

Acceptance:

- Automated template smoke covers open/edit/save/reload and supported play paths.
- Manual or screenshot evidence proves the templates work in the app window.

### E11: Script Authoring Baseline

Goal: scripting is visible and debuggable in Studio without hard-coding the
engine forever to one language.

Required behavior:

- Attach, detach, and inspect existing Lua/Luau script assets through the UI.
- Show script lifecycle and diagnostics in Studio.
- Keep generic editor-facing names such as script runtime, script asset, and
  script diagnostics where APIs are language-neutral.
- File scoped follow-ups for `.kn` recognition, syntax support, diagnostics,
  lifecycle callbacks, async/task behavior, HTTP permissions, and hot reload once
  `kinetik-lang` integration is ready.

Acceptance:

- Existing Lua/Luau behavior remains intact.
- Script UI tests or manual smoke cover attach/detach and diagnostics.
- No accepted scripting ADR is redirected without a new approval path.

### E12: Build, Run, and Export Prototype

Goal: a simple project can leave the editor through a documented path.

Required behavior:

- Provide a UI command to validate and build or export the active project through
  the current bundle/runtime plan.
- Produce deterministic output with manifest diagnostics.
- Launch or run the exported output when supported by the platform.
- Report missing assets, scripts, or runtime requirements before export.

Acceptance:

- Fixture tests cover deterministic output for a minimal project.
- Manual smoke proves the UI command runs and reports success/failure clearly.

### E13: Editor Reliability Pass

Goal: make the editor feel like software someone can keep open.

Required behavior:

- Persist layout and recent-project state.
- Add stable keyboard shortcuts and menu affordances for core actions.
- Add diagnostics/log filtering.
- Add screenshot or visual smoke coverage for the main editor states.
- Profile obvious UI stalls and eliminate avoidable busy loops.
- Audit oversized/mixed-responsibility editor files and file follow-ups or split
  them when the work belongs to the active slice.

Acceptance:

- Main editor states have reproducible smoke coverage.
- Known reliability issues have scoped issues with acceptance criteria.

## Subsystem Roadmap After The Usable Editor Spine

The following feature families are deferred until the editor spine above is
usable, unless a narrow issue is required as a dependency for an earlier editor
milestone.

### Audio

Target capabilities:

- Audio asset records, import metadata, missing-file diagnostics, and project
  validation.
- Scene components for audio emitters/listeners.
- Runtime audio service with deterministic lifecycle and editor-safe preview.
- UI controls for assigning clips and previewing basic playback.

Gate: requires an ADR or accepted internal spec for backend choice, threading,
streaming, asset formats, and editor/runtime ownership.

### Animation

Target capabilities:

- Animation clip assets and skeleton/skin metadata where supported.
- Animator component and state/montage model.
- Runtime evaluation tied to frame phases.
- Editor timeline or minimal animation preview once viewport selection works.

Gate: requires accepted data model and importer scope before adding broad
animation APIs.

### Runtime UI

Target capabilities:

- Runtime UI document/assets, layout model, styling subset, and input routing.
- Editor preview and diagnostics for invalid UI documents.
- Bridge between runtime UI and script callbacks without locking to one language.

Gate: requires a UI architecture decision before serialized formats are added.

### Prefabs and Packages

Target capabilities:

- Prefab asset format, instance overrides, nested prefab policy, and validation.
- Package manifests for reusable content.
- Editor create/apply/revert flows with clear ownership of overrides.

Gate: requires accepted serialization and override semantics.

### Build, Export, and Bundles

Target capabilities:

- `.knbundle` or equivalent project output.
- Platform runtime manifests and asset closure validation.
- Incremental build cache if justified by measured project scale.
- Editor export UI and command-line export parity.

Gate: export format changes require explicit approval and migration policy.

### Terrain and Worlds

Target capabilities:

- Terrain asset records, height/paint data, streaming policy if needed.
- Editor sculpt/paint tools only after viewport tools are reliable.
- Runtime chunk/render integration and diagnostics.

Gate: requires an accepted world/terrain data model and performance budget.

### Advanced Rendering

Target capabilities:

- Expanded material model, post-processing, shadow quality controls, and render
  settings assets.
- Shader authoring or shader graph only after renderer/editor boundaries are
  stable.
- Visual diagnostics for unsupported material/render paths.

Gate: public renderer API or serialized material changes need accepted ADR/spec
coverage.

### Asset Pipeline Expansion

Target capabilities:

- Reimport tracking, dependency graph, thumbnails, cache invalidation, and
  project health checks.
- Importer-specific diagnostics and source-control-friendly metadata.

Gate: no new importer family should be added without fixture coverage and clear
  editor UX acceptance.

### Scripting Maturity

Target capabilities:

- Hot reload, permission diagnostics, async/task behavior, HTTP permissions, and
  editor debugger hooks for current Lua/Luau support.
- Future `.kn` asset recognition, syntax support, diagnostics, lifecycle, and
  runtime adapter once `kinetik-lang` is ready.

Gate: multi-language scripting is additive. Do not replace or disrupt accepted
  Lua/Luau architecture without human approval.

### Physics Maturity

Target capabilities:

- Collider/rigid-body editing, debug drawing, simulation controls, and runtime
  determinism checks.
- Physics fixture coverage for templates that rely on character motion.

Gate: dependency/backend changes require approval.

### Networking

Target capabilities:

- Project settings, runtime service boundaries, diagnostics, and local test
  harnesses before gameplay-facing networking APIs.

Gate: requires explicit architecture approval before public APIs or serialized
  protocols are added.

### 2D Workflow

Target capabilities:

- Sprite/atlas assets, 2D camera presets, tilemaps if approved, and 2D template
  projects.

Gate: must reuse the editor spine instead of becoming a separate toy editor.

## Issue Creation Order

For each milestone, prefer issues in this order:

1. Small spec or ADR issue when architecture, public API, serialized format, or
   dependency choice is not already approved.
2. Deterministic model/command/data issue with automated tests.
3. Editor UI integration issue with manual or screenshot acceptance.
4. Template or fixture verification issue proving the workflow end to end.
5. Reliability/code-health follow-up if the implementation exposes oversized
   files, unclear ownership, hard-to-test helpers, excessive nesting, or
   duplication outside the active slice.

Do not create broad "implement subsystem" issues. Every issue should name the
files or crates it is likely to touch, the user-visible behavior it unlocks, the
required checks, and what will still be out of scope.
