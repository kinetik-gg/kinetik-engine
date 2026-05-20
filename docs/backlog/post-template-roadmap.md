# Post-Template Feature Roadmap

Status: planning backlog for work after the first 3D template set.

This document records major Kinetik engine and editor capabilities that remain
planned after the primitive showcase, PBR material demo, and basic FPS prototype.
It is a backlog map, not implementation approval. Each feature family below
needs a scoped GitHub issue before implementation starts, and architecture-
sensitive work needs the listed dependency review, ADR, or internal API gate.

## First-Template Boundary

The first template set proves that Kinetik can author, save, reload, inspect,
render, and run deterministic headless 3D loops through the current editor and
runtime foundations.

The first template set does not require:

- Real-time GPU viewport gameplay beyond the staged renderer path already
  covered by rendering milestones.
- Platform keyboard/mouse backends, OS pointer capture, or polished input UX.
- Dynamic physics, joints, triggers, areas, collision-layer UI, or debug draw.
- Script-authored gameplay, debugger hooks, hot reload, or generated API docs.
- Audio playback, spatial audio, runtime UI, animation, terrain editing,
  packages, bundles, build/export, networking, or 2D templates.

Future work must not retroactively expand the acceptance scope of the completed
templates. If a later feature needs a template refresh, create a separate issue
with explicit verification changes.

## Roadmap Gates

Every post-template implementation issue must name:

- Owning crate or document area.
- Implementation level and required checks.
- Relevant ADRs and internal API specs.
- Whether public APIs, serialized source, generated contracts, migrations,
  dependency boundaries, editor/runtime boundaries, or scripting direction are
  affected.
- Verification plan: unit, golden, integration, headless/MCP, screenshot, or
  human review.

Work must stop for human approval when it needs a new dependency, unsafe Rust,
public API change, serialized format change, migration, editor/runtime boundary
change, or new architecture decision not already covered by accepted ADRs.

## Deferred Feature Families

### Audio

Goal: make audio a first-class runtime and editor system.

Planned capabilities:

- Audio buses and mixer hierarchy.
- Playback sources attached to instances.
- Spatial audio and attenuation policies.
- Editor preview controls and diagnostics.
- Import/validation for audio assets.
- Mixing workflows for templates and exported games.

Gates before implementation:

- Dependency proposal or update for the chosen audio backend, including Kira
  boundary ownership, license, platform support, unsafe/FFI exposure, and build
  impact.
- Internal API spec for audio runtime state, scene/reflection integration,
  diagnostics, and editor preview behavior.
- Frame-order alignment with ADR 0022 so audio observes coherent runtime state.

Acceptance criteria for first issue:

- A minimal audio model is engine-owned and does not leak backend types.
- Missing/invalid audio assets produce structured diagnostics.
- Editor preview state cannot mutate saved project state outside commands.

### Animation

Goal: support authored and imported animation without coupling it directly to
the renderer, physics, or editor UI.

Planned capabilities:

- Animation clip assets and import metadata.
- Skeletal import direction for GLB/glTF.
- Animation state playback on instances.
- Retargeting direction and compatibility diagnostics.
- Editor preview for clips and pose state.

Gates before implementation:

- Import/cache extension plan for animation artifacts and dependency graphs.
- Internal API spec for clip identity, playback state, frame-order updates, and
  diagnostics.
- Renderer and scene agreement on transform/skeleton extraction boundaries.

Acceptance criteria for first issue:

- Clip metadata can be represented without changing unrelated source formats.
- Runtime playback order is deterministic and tied to ADR 0022.
- Editor preview has an explicit edit/play ownership model.

### Runtime UI

Goal: add game-facing UI that participates in runtime input, layout, and
diagnostics while staying separate from editor chrome.

Planned capabilities:

- UI scene instances and templates under the `UI` service.
- Layout, text, image, focus, and input routing.
- Styling direction and theme inheritance.
- Menu and HUD workflows.
- Editor preview and runtime diagnostics.

Gates before implementation:

- Internal API spec for runtime UI tree, focus, layout, input ownership, and
  diagnostics.
- Renderer/editor boundary decision for using runtime renderer output versus
  editor UI toolkit primitives.
- Scripting API review for Luau-facing UI services without blocking future
  multi-language script backends.

Acceptance criteria for first issue:

- UI state is runtime-owned and inspectable without depending on editor crates.
- Input focus behavior is deterministic and testable headlessly where possible.
- Editor preview does not bypass command/change-record boundaries.

### Prefabs And Packages

Goal: make reusable content inspectable, clone-ready, updateable, and safe for
Git review and agent automation.

Planned capabilities:

- Serialized prefab assets.
- Clone-ready prefab workflows.
- Explicit override records and broken-override diagnostics.
- Package manifests and package dependency metadata.
- Package updates that preserve local edits or explain why they cannot.

Gates before implementation:

- ADR 0016 remains the governing prefab/package override direction.
- Serialized prefab/source format issue with golden fixtures and migration notes
  if needed.
- Command/MCP spec updates for applying, reverting, unpacking, and updating
  overrides through validated editor commands.

Acceptance criteria for first issue:

- Prefab relationships and overrides are explicit source data.
- Broken overrides produce stable diagnostics.
- MCP/editor operations share command and semantic change paths.

### Build, Export, And Bundles

Goal: produce generated runtime content packages and exportable game builds
without confusing source assets, cache, and distributable output.

Planned capabilities:

- `.knbundle` build/load/inspect/verify flow.
- Bundle manifests, dependency graphs, hashes, and optional signatures.
- Runtime content mounting.
- Platform export pipeline.
- Signing, hash verification, and compatibility diagnostics.

Gates before implementation:

- ADR 0009 governs `.knbundle` direction.
- Dependency/security review for hashing/signing libraries before installation.
- Internal API spec for bundle contents, resource mounting, diagnostics, and
  source/cache/build separation.

Acceptance criteria for first issue:

- Bundle output is generated and never confused with committed source state.
- Verification diagnostics are stable and attributable.
- Runtime loading cannot bypass resource identity or permission policy.

### Terrain And World Environment

Goal: give 3D projects richer world defaults and terrain authoring without
turning early templates into terrain demos.

Planned capabilities:

- `Workspace.Terrain` object and terrain chunks.
- Heightmap terrain first, with voxel terrain left as future direction.
- Brushes, editor tools, and terrain diagnostics.
- Sky, atmosphere, time-of-day, weather direction, and large-world constraints.

Gates before implementation:

- ADR 0010 governs terrain/world direction.
- Internal API spec for terrain storage, edit operations, runtime extraction,
  serialization impact, and editor tooling.
- Dependency proposal if terrain storage, noise, or compression needs external
  crates.

Acceptance criteria for first issue:

- Terrain appears through instance-based authoring where user-facing.
- Serialized terrain data has an explicit deterministic format plan.
- Editor tools use commands and do not write private editor-only state.

### Advanced Rendering

Goal: grow the renderer from the first visible subset into the accepted Kinetik
rendering direction.

Planned capabilities:

- Shadows, HDR, tone mapping, and color correctness.
- Environment lighting and image-based lighting.
- Render graph organization.
- Material graph and shader graph authoring.
- Generated WGSL surface functions.
- Forward+/clustered lighting direction.

Gates before implementation:

- ADR 0006 governs renderer and shader graph direction.
- Dependency proposal updates before renderer crate additions or upgrades.
- Internal API specs for render extraction, shader/material graph IR,
  diagnostics, and editor viewport integration.

Acceptance criteria for first issue:

- Runtime scene state and GPU state remain separated by extraction boundaries.
- Authoring APIs do not expose raw `wgpu` types.
- Rendering failures produce structured diagnostics.

### Asset Pipeline Expansion

Goal: broaden asset import and repair while preserving source-of-truth identity
and deterministic project state.

Planned capabilities:

- Richer model/material import.
- Texture compression and platform profiles.
- Reimport policies and cache invalidation.
- Repair tools for missing, moved, or incompatible assets.
- Asset dependency visualization.

Gates before implementation:

- Dependency proposal updates for importers, compression, and image/model
  libraries.
- Resource database and import-cache internal API review.
- Golden fixtures for any manifest, cache metadata, or generated contract
  changes.

Acceptance criteria for first issue:

- Source assets remain project-owned and cache output remains disposable.
- Resource identity uses stable GUID plus `res://` path.
- Diagnostics explain repair choices instead of silently replacing identity.

### Editor Polish

Goal: make Kinetik Studio ergonomic and trustworthy for repeated authoring work.

Planned capabilities:

- Docking and layout persistence.
- Keyboard shortcuts and command palette direction.
- Accessibility and focus management.
- Profiling panels and diagnostics affordances.
- Visual regression harness.
- Richer viewport gizmos and overlays.

Gates before implementation:

- Editor UI toolkit decision or fallback review where toolkit maturity affects
  behavior.
- Internal API spec for persisted editor-local state versus project source
  state.
- Screenshot/visual regression strategy before broad visual polish work.

Acceptance criteria for first issue:

- Editor-local state cannot leak into project source files.
- UI actions mutate project state only through commands.
- Visual checks are reproducible enough for review.

### Scripting Maturity

Goal: deepen scripting while keeping current Lua/Luau support intact and
planning for future Kinetik `.kn` scripts side by side.

Planned capabilities:

- Luau type generation from reflection metadata.
- Debugger hooks and runtime traceability.
- Hot reload policy and lifecycle diagnostics.
- Script API documentation.
- Sandbox permissions and HTTP policy tooling.
- Future `.kn` asset recognition, syntax support, diagnostics, lifecycle,
  async/task behavior, HTTP permissions, and hot reload after `kinetik-lang` is
  ready.

Gates before implementation:

- ADR 0008 remains the accepted Luau scripting direction.
- Any `.kn` integration needs a focused ADR or internal API spec before code.
- Permissioned HTTP work must follow ADR 0021 provenance and policy.
- Runtime/frame work must follow ADR 0020 and ADR 0022.
- Generic engine APIs should remain language-neutral when they are not
  explicitly Luau bridge APIs.

Acceptance criteria for first issue:

- Luau-specific behavior stays in Luau-owned surfaces.
- Generic script lifecycle, diagnostics, assets, permissions, and hot reload
  contracts can identify a script backend without assuming there is only one.
- No accepted Luau API or serialized script contract is changed without focused
  approval.

### Physics Maturity

Goal: grow from the deterministic controller/static collision foundation toward
instance-authored physics.

Planned capabilities:

- Rapier-backed rigid bodies and colliders.
- Joints, triggers/areas, and collision events.
- Collision layers and editor UI.
- Character-controller refinement.
- Debug drawing and editor quick fixes.

Gates before implementation:

- ADR 0007 governs instance-based Rapier-backed physics.
- Dependency proposal update before adding or upgrading Rapier.
- Internal API spec for body/collider ownership, event timing, diagnostics, and
  scene/reflection integration.
- Frame-order alignment with ADR 0022.

Acceptance criteria for first issue:

- Physics authoring remains instance-based and system-owned.
- Scripts and editor code do not receive raw Rapier types.
- Collider/body validation failures produce actionable diagnostics.

### Networking And Multiplayer

Goal: defer networking until the local runtime, scripting, diagnostics, and
editor workflows are mature enough to support it deliberately.

Planned capabilities:

- Network model decision.
- Replication or session architecture if product direction requires it.
- Diagnostics and tooling for authority, synchronization, and errors.

Gates before implementation:

- New ADR required before any networking/multiplayer implementation.
- Runtime execution and scripting maturity must be proven locally first.
- Permission and security review required before exposing network capability to
  scripts or exported games.

Acceptance criteria for first issue:

- The issue is planning/ADR work, not implementation.
- It identifies authority, replication, diagnostics, and security boundaries.
- It does not reinterpret Kinetik as a Roblox hosted-server replacement.

### 2D Support

Goal: keep 2D support as future work while Kinetik remains optimized for the
first 3D milestones.

Planned capabilities:

- 2D scene classes and transforms.
- 2D rendering and materials.
- 2D physics and UI/input integration.
- 2D templates after 3D authoring and runtime foundations are stable.

Gates before implementation:

- New or amended ADR required to define 2D scope and its relationship to the
  3D-first runtime.
- Internal API review for shared versus separate 2D/3D reflection, rendering,
  physics, and editor tools.
- Dependency review if 2D physics or rendering needs new crates or features.

Acceptance criteria for first issue:

- 2D work remains additive and does not weaken the 3D-first architecture.
- Shared systems are explicit rather than inferred through vague abstractions.
- First 2D templates are not opened until the 2D source/runtime contracts exist.

## Issue Creation Order

When the loop resumes after M32, prefer issues that unlock one coherent feature
family at a time:

1. Planning/spec issue for the chosen family.
2. Dependency proposal issue if external crates or features are needed.
3. Small implementation issue for the lowest-risk deterministic contract.
4. Integration issue connecting editor, runtime, MCP, diagnostics, or templates.
5. Verification issue for screenshot, fixture, or human review gaps.

Do not open broad “implement subsystem” issues. Each issue should remain small
enough for one branch, one PR, and one clear definition of done.
