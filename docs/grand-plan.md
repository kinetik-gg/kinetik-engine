# Kinetik Engine Grand Plan

**Version:** 0.1 Grand Plan  
**Status:** Foundational architecture and execution constitution  
**Horizon:** Marathon project, not sprint project  
**Primary audience:** Luki, Kinetik architects, AI agent swarm, future contributors  

---

## 1. Executive Summary

Kinetik Engine is a modern, creator-friendly, high-performance game engine built around four core pillars:

1. **Rust-native core** for safety, performance, portability, and maintainability.
2. **Luau scripting** for approachable, typed, Roblox-familiar gameplay development.
3. **Instance-based scene architecture** inspired by Godot and Roblox Studio, optimized for intuitive authoring.
4. **Editor experience** with Blender-grade interaction quality, Vello-powered custom UI, shader graph authoring, and clean project/source-control workflows.

The project is intentionally ambitious. The goal is not to cut the dream down to a weekend MVP. The goal is to make the dream decomposable, governable, and executable by a swarm of specialized AI agents without the codebase becoming incoherent.

Kinetik is not trying to become a Roblox hosting platform. It borrows Roblox Studio's organizational clarity — live workspace, clone-ready prefabs, services, approachable scripting — but remains a general-purpose engine where users own their projects, assets, builds, hosting, and distribution.

The core product thesis:

> Kinetik gives creators Roblox-like approachability, Godot-like scene clarity, Blender-like authoring power, and Rust-native engine foundations.

The key execution thesis:

> Autonomous agent execution is allowed, but architectural direction, public API, dependencies, serialized formats, and merges remain supervised.

---

## 2. Non-Negotiable Identity

### 2.1 What Kinetik Is

Kinetik is:

- A modern Rust-native game engine.
- A creator-friendly Studio-style environment.
- A instance-based authoring engine.
- A Luau-scripted runtime.
- A practical PBR renderer with graph-authored materials.
- A Git-friendly project system.
- A long-horizon platform designed for agent-assisted development.

### 2.2 What Kinetik Is Not

Kinetik is not:

- A Roblox hosted-server replacement.
- A clone of Roblox APIs.
- A pure ECS engine exposed directly to users.
- A renderer-first tech demo.
- A giant monolithic editor binary with hidden state.
- A project where AI agents freely rewrite everything.
- A sprint-driven MVP that sacrifices the final architecture.

### 2.3 Product Positioning

Kinetik should eventually feel like:

- **Roblox Studio clarity** for hierarchy, prefab/template workflows, and approachability.
- **Godot structure** for instance-based scenes and clean lifecycle.
- **Blender creativity** for shader graphs, editor polish, and visual authoring.
- **Unity-like practicality** for materials, prefabs, scenes, assets, and builds.
- **Rust-native reliability** for engine internals.

---

## 3. Core Technology Decisions

These decisions are foundational unless reopened through an Architecture Decision Record.

| Area | Decision | Rationale |
| --- | --- | --- |
| Core language | Rust | Safety, performance, strong ecosystem, long-term maintainability |
| Scripting language | Luau | Optional typing, Roblox familiarity, better editor tooling, creator scale |
| Renderer API | wgpu | Cross-platform modern GPU abstraction over Metal/Vulkan/DX/WebGPU |
| Shader language | WGSL generated/managed by engine | Native to wgpu/WebGPU direction; controlled shader pipeline |
| Editor UI | Vello long-term | Custom UI, GPU vector rendering, Blender-grade interaction aspiration |
| Physics | Rapier | Rust-native, fast, 2D/3D capable |
| Window/input | winit | Rust-native cross-platform window and event handling |
| Audio | Kira preferred, Rodio acceptable for simple playback | Kira offers richer game-audio design |
| Project files | Text-first + stable GUIDs | Git-friendly, diffable, mergeable |
| Runtime bundles | `.ktbundle` | Generated runtime content packages for local/remote loading |
| Source assets | committed to Git where reasonable | Source of truth should travel with project |
| Import cache | not committed | Rebuildable, generated, platform-specific/noisy |

---

## 4. Architecture Overview

Kinetik is divided into clear layers and crates. The purpose is not bureaucracy. The purpose is to let many agents build independently without producing architectural soup.

### 4.1 Core Layers

```text
Kinetik Studio / Editor
  ↓ uses
Kinetik Runtime / App
  ↓ uses
Scene, Resources, Script, Render, Physics, Audio, UI, Terrain, Bundles
  ↓ use
Core primitives, errors, handles, math, serialization
```

### 4.2 Proposed Workspace Layout

```text
kinetik/
  Cargo.toml
  README.md
  AGENTS.md

  docs/
    constitution.md
    architecture/
    adr/
    agents/
    quality/

  crates/
    kinetik-core/
    kinetik-app/
    kinetik-scene/
    kinetik-resource/
    kinetik-render/
    kinetik-script/
    kinetik-script-luau/
    kinetik-signal/
    kinetik-physics/
    kinetik-audio/
    kinetik-reflect/
    kinetik-editor/
    kinetik-ui/
    kinetik-terrain/
    kinetik-bundle/
    kinetik-test/

  examples/
    hello_scene/
    baseplate/
    shader_graph_material/

  tools/
    agent-loop/
```

### 4.3 Dependency Direction Rules

Allowed:

```text
kinetik-editor -> kinetik-runtime/app
kinetik-app -> scene/render/script/physics/audio/resource
kinetik-script-luau -> kinetik-script + core APIs
kinetik-render -> resource/core/math
kinetik-physics -> scene/core/math
```

Forbidden:

```text
kinetik-core -> kinetik-editor
kinetik-scene -> kinetik-editor
kinetik-render -> kinetik-editor
kinetik-script -> concrete editor systems
kinetik-physics -> concrete editor systems
```

The runtime must not depend on the editor. Editor state and game state must be separate.

---

## 5. Agent Constitution

This section is mandatory context for every AI agent working on Kinetik.

### 5.1 Prime Directive

> Build Kinetik as a cohesive, polished, maintainable game engine. Every contribution must improve the engine without damaging architectural clarity, editor quality, runtime stability, or future extensibility.

Agents are not here to generate code volume. Agents are here to ship correct, elegant, integrated systems.

### 5.2 No Silent Assumptions

Agents must not invent major behavior. If an assumption affects architecture, public API, serialization, editor UX, scripting API, performance model, dependency choices, or long-term maintenance, the agent must stop and ask.

Allowed assumptions:

- Tiny local implementation details.
- Obvious defaults inside a private function.
- Formatting choices already defined by project style.

Forbidden assumptions:

- New public APIs.
- New serialized formats.
- New dependencies.
- New architecture patterns.
- Changes to user-facing behavior.
- Reinterpretation of assigned scope.

### 5.3 If in Doubt, Ask With Options

Agents should not ask lazy questions. They should present viable options and recommend one.

Bad:

> How should I implement this?

Good:

> There are two viable designs. A keeps Luau state external and simplifies hot reload. B embeds state into instances but complicates invalidation. I recommend A. Should I proceed?

### 5.4 Do Not Touch Unrelated Code

Forbidden unless explicitly assigned:

- Broad refactors.
- Formatting unrelated files.
- Renaming unrelated APIs.
- Dependency upgrades.
- Moving files across crates.
- “While I’m here” cleanup.

Rule:

> Leave the codebase cleaner inside the area you were asked to touch. Do not wander.

### 5.5 Small, Focused Patches

Every patch should be reviewable. One task, one concept, one bounded subsystem.

Preferred:

- Clear scope.
- Tests.
- Explicit assumptions.
- No hidden behavior changes.
- No mixed feature/refactor patches.

### 5.6 Good Taste Is Required

Good taste means:

- Simple things stay simple.
- Hard things are isolated.
- Names are obvious.
- APIs are hard to misuse.
- Errors explain failure.
- No cleverness for its own sake.
- No abstractions without pressure.
- No fake completeness.
- No “AI slop” comments.

### 5.7 Code Quality Laws

1. **No god files.** Files above roughly 500–700 lines need scrutiny.
2. **No god objects.** Top-level coordinators may coordinate; they must not absorb all logic.
3. **Avoid 3+ nesting.** Use guard clauses and extracted functions.
4. **Use typed IDs.** `InstanceId`, `ResourceId`, `SignalId`, not interchangeable `u64`.
5. **No vague utilities.** Avoid `utils.rs`, `helpers.rs`, `misc.rs` unless truly justified.
6. **Errors must be structured and meaningful.**
7. **No premature abstraction.** Traits require real pressure.
8. **Public APIs require docs, tests, and review.**
9. **Serialization is sacred.** Format changes require migration/versioning.
10. **Runtime/editor boundaries are sacred.**

---

## 6. AI Swarm Operating Model

### 6.1 Role Types

#### Chief Architect Agent

Owns:

- Architecture documents.
- Crate boundaries.
- ADRs.
- Dependency approval.
- Naming consistency.
- Public API review.

This agent shapes systems more than it writes code.

#### Domain Builder Agents

Specialized implementers:

- Scene Graph Agent.
- Render Agent.
- Luau Bridge Agent.
- Physics Agent.
- Resource Agent.
- Editor Agent.
- UI Agent.
- Terrain Agent.
- Bundle Agent.
- Audio Agent.
- Shader Graph Agent.

Each owns one bounded domain and must not redesign the entire engine.

#### Integration Agent

Owns vertical slices and cross-system validation.

Example mission:

> Create a scene, attach a Luau script, spawn a prefab, move a physics character, render it, and save/reload the scene.

#### QA/Test Agent

Adversarial role. Owns:

- Regression tests.
- Golden fixtures.
- Invalid handle tests.
- Serialization roundtrips.
- Hot reload failure cases.
- Physics determinism checks.
- Renderer fallback behavior.

#### Refactor Janitor Agent

Only works when explicitly assigned. Reduces duplication, splits god files, simplifies nested code. No new features.

#### Taste/UX Agent

Owns feel:

- Editor interaction quality.
- Naming from creator perspective.
- Shader graph usability.
- Inspector clarity.
- Defaults and templates.
- “Does this feel polished, coherent, and useful?”

#### Documentation Agent

Owns:

- Contributor docs.
- API docs.
- Tutorials.
- Examples.
- Architecture maps.

### 6.2 Task Template

Every agent task must include:

```text
Mission:
Scope:
Do not touch:
Inputs:
Expected outputs:
Required tests/checks:
Relevant docs:
Open questions:
Definition of done:
```

### 6.3 Autonomous Loop With Codex CLI

Kinetik may use a supervised autonomous loop:

```text
Backlog
  ↓
Architect selects task
  ↓
Worker creates branch/worktree
  ↓
Codex CLI implements scoped task
  ↓
Checks run
  ↓
Reviewer/QA agent attacks patch
  ↓
Integration agent validates if needed
  ↓
Patch/PR produced
  ↓
Human or Architect approves
  ↓
Merge
  ↓
Docs/ADR updated
```

Agents may implement and test. They may not merge to main, add dependencies, change public API, alter serialization, or broaden scope without approval.

### 6.4 Worktree Rule

Each active coding agent gets its own worktree/branch.

```text
worktrees/
  agent-scene-tree/
  agent-signal-bus/
  agent-resource-cache/
```

No multiple agents editing the same working directory.

---

## 7. Project Organization Model

Kinetik borrows Roblox Studio’s clarity but removes hosted client/server assumptions.

### 7.1 Editor Hierarchy

Default project hierarchy:

```text
Game
  Workspace
  Prefabs
  Scripts
  UI
  Lighting
  Audio
  Physics
  Assets
  Packages
```

### 7.2 Meaning

#### Workspace

Live objects in the active world.

```text
Workspace
  Terrain
  Baseplate
  Camera
  Map
  Player
  Enemies
  Pickups
```

If it is visible, physical, simulated, or active in the scene, it usually lives here.

#### Prefabs

Inactive clone-ready scene instances.

```text
Prefabs
  Coin
  Enemy
  Bullet
  Door
  ExplosionEffect
```

Luau example:

```lua
local enemy = game.Prefabs.Enemy:clone()
enemy.position = Vec3.new(0, 2, 0)
game.Workspace.Enemies:add_child(enemy)
```

#### Scripts

Global runtime systems/managers.

```text
Scripts
  GameManager.luau
  EnemySpawner.luau
  SaveSystem.luau
```

Object-specific scripts stay attached to instances.

#### UI

Runtime UI roots/templates.

```text
UI
  HUD
  PauseMenu
  Inventory
  MainMenu
```

#### Lighting

Global world lighting/time/sky/atmosphere service.

#### Audio

Global audio buses/mixers.

#### Physics

Global gravity, collision layers, solver settings.

#### Assets

File-backed resources: models, textures, materials, audio, shaders, animations, fonts.

#### Packages

Reusable local/external content and code modules.

### 7.3 Git-Friendly Project Layout

```text
my_game/
  Kinetik.toml

  scenes/
    main.ktscene

  prefabs/
    Coin.ktprefab
    Enemy.ktprefab

  scripts/
    GameManager.luauu

  project/
    assets.ktmanifest
    instances.ktmanifest

  assets/
    models/
    textures/
    materials/
    audio/
    shaders/
    animations/

  packages/

  .kinetik/
    cache/       # ignored
    imported/    # ignored
    build/       # ignored
```

### 7.4 Source Control Rules

Must:

- Text-first scene/prefab/material/project files.
- Stable GUIDs for assets and scene instances.
- Deterministic serialization order.
- No random timestamps in saved files.
- No absolute machine-local paths.
- Import cache ignored.
- Build bundles ignored unless explicitly published.

---

## 8. Scene, Instances, Objects, Primitives, and Imported Models

### 8.1 Core Rule

> A game object is an Instance. A mesh is a Resource. A visible object is an Instance with a MeshRenderer referencing a Mesh resource.

### 8.2 Scene Object Model

```text
Instance3D
  Transform
  children
  scripts
  components/instances
```

Visible object:

```text
Cube
  MeshRenderer3D(mesh = builtin://mesh/cube)
```

Imported object:

```text
Tree
  MeshRenderer3D(mesh = res://assets/models/tree.glb#mesh/Trunk)
```

### 8.3 Built-In Primitives

Provide built-in primitive mesh resources:

```text
builtin://mesh/cube
builtin://mesh/sphere
builtin://mesh/capsule
builtin://mesh/cylinder
builtin://mesh/cone
builtin://mesh/plane
builtin://mesh/quad
```

Editor actions:

```text
Add → 3D Object → Cube
Add → Physics → Rigid Cube
```

Rigid Cube creates:

```text
RigidCube
  RigidBody3D
  BoxCollider3D
  MeshRenderer3D(mesh = builtin://mesh/cube)
```

### 8.4 Imported Meshes

Kinetik should treat GLB/glTF as first-class. FBX can come later, ideally through conversion or a dedicated importer.

Priority:

1. `.glb` / `.gltf` first-class.
2. `.obj` simple static mesh fallback.
3. `.fbx` later.

### 8.5 Model Resource

A GLB may contain a full scene. Import as a ModelAsset:

```text
ModelAsset
  scenes
  instances
  meshes
  materials
  textures
  skeletons
  animations
```

Dragging `house.glb` into Workspace can create:

```text
House
  ImportedScene3D(source = res://assets/models/house.glb)
    Walls
      MeshRenderer3D
    Door
      MeshRenderer3D
```

### 8.6 Linked vs Unpacked

Support both:

- **Linked instance:** updates when source model changes.
- **Unpacked scene:** user can edit children freely.

Editor actions:

```text
Right click → Unpack
Right click → Make Local
Right click → Reimport
```

### 8.7 Gameplay Pattern

Imported model is usually visuals, not the gameplay root.

```text
Enemy
  CharacterBody3D
  CapsuleCollider3D
  EnemyAI.luau
  Visuals
    ImportedScene3D(source = res://assets/models/enemy.glb)
```

---

## 9. Resource and Asset System

### 9.1 Asset Categories

```text
Source Assets        → committed to Git where reasonable
Central manifests    → committed
Imported Cache       → ignored
Build Bundles        → ignored/generated unless publishing
Remote/Large Assets  → optional registry/LFS/cloud
```

### 9.2 Source Assets

Usually committed:

```text
assets/models/tree.glb
assets/textures/grass.png
assets/audio/jump.wav
assets/materials/lava.kmat
assets/shaders/lava.kgraph
scripts/player.luau
```

These are the source of truth.

### 9.3 Imported Cache

Ignored:

```text
.kinetik/cache/
.kinetik/imported/
.kinetik/build/
```

### 9.4 Project Manifests

Source assets should have stable metadata in centralized manifests:

```text
assets/models/tree.glb
project/assets.ktmanifest
```

Example:

```toml
[[assets]]
guid = "asset_8f4b..."
path = "res://assets/models/tree.glb"
importer = "gltf"
version = 1

[assets.settings]
scale = 1.0
generate_tangents = true
import_materials = true
generate_collision = "none"
```

### 9.5 Git LFS

Kinetik Studio should offer to configure Git LFS for large binary assets:

```gitattributes
*.png filter=lfs diff=lfs merge=lfs -text
*.jpg filter=lfs diff=lfs merge=lfs -text
*.wav filter=lfs diff=lfs merge=lfs -text
*.glb filter=lfs diff=lfs merge=lfs -text
*.fbx filter=lfs diff=lfs merge=lfs -text
*.blend filter=lfs diff=lfs merge=lfs -text
```

### 9.6 References

Serialized scene files reference logical paths, not cache files.

Good:

```toml
mesh = "res://assets/models/tree.glb#mesh/Trunk"
material = "res://assets/materials/bark.kmat"
```

Bad:

```toml
mesh = ".kinetik/imported/abc123.ktmesh"
```

---

## 10. Bundle System

Kinetik should provide generated runtime content bundles similar in spirit to Unity AssetBundles, but designed cleanly from the start.

### 10.1 Bundle Concept

```text
Source assets
  ↓ import
Optimized runtime assets
  ↓ pack
.ktbundle
  ↓ load from disk/CDN/S3/mod folder
Mounted runtime resources
```

Example bundles:

```text
base_game.ktbundle
chapter_02.ktbundle
weapons.ktbundle
winter_event.ktbundle
world_chunk_0_0.ktbundle
```

### 10.2 Uses

- DLC.
- Live events.
- Streaming worlds.
- Cloud-hosted content.
- Mods/user-generated content.
- Patchable game updates.

### 10.3 Bundle Contents

A bundle can contain:

- Meshes.
- Textures.
- Materials.
- Shader graphs/generated shaders.
- Audio.
- Animations.
- Prefabs.
- Scenes.
- UI.
- Luau scripts, if trusted/allowed.
- Manifest/dependency graph.

### 10.4 Security Classes

#### Content-only bundle

No executable scripts. Safer for remote loading.

#### Trusted gameplay bundle

May include Luau scripts. Must be from trusted/signed source.

#### Mod bundle

May include sandboxed scripts under explicit project policy.

### 10.5 Signing and Verification

Cloud-loaded bundles should support:

- SHA-256 hashes.
- Ed25519 signatures.
- Engine version compatibility.
- Bundle version.
- Optional encryption.

Runtime order:

```text
download
verify hash
verify signature if required
check compatibility
mount
resolve assets
```

### 10.6 Addressables

Avoid hardcoding bundle URLs in gameplay scripts.

```lua
local prefab = assets.load_async("enemy.ice_golem")
```

Address map:

```toml
[addressables]
"enemy.ice_golem" = {
  guid = "asset_abc",
  bundle = "chapter_02",
  path = "res://game/prefabs/IceGolem.ktprefab"
}
```

### 10.7 CLI

```bash
kinetik bundle build --config bundles.toml
kinetik bundle inspect dist/chapter_02.ktbundle
kinetik bundle verify dist/chapter_02.ktbundle
kinetik bundle upload s3://my-game-content/
```

---

## 11. Renderer and Shader System

### 11.1 Renderer Philosophy

> The engine owns rendering structure. Users own material expression.

Kinetik should start with a practical standard PBR renderer comparable in spirit to Roblox modern materials and Unity’s earlier/URP-style renderer, not Unreal-level cinematic rendering.

### 11.2 Initial Renderer

Target:

- Forward renderer first.
- Forward+ later.
- Metallic/roughness PBR.
- HDR internal target.
- Tone mapping.
- Gamma correctness.
- Directional light.
- Point lights.
- Normal maps.
- Emissive.
- Environment lighting later.
- Shadows after basic PBR.

### 11.3 Standard Material

```text
StandardMaterial
  base_color: Color
  metallic: f32
  roughness: f32
  normal_texture
  base_color_texture
  metallic_roughness_texture
  ao_texture
  emissive
  alpha_mode
```

### 11.4 PBR Model

Use glTF-style metallic/roughness:

- Cook-Torrance.
- GGX distribution.
- Smith geometry.
- Schlick Fresnel.
- Lambert diffuse.

### 11.5 Shader Graph Direction

Kinetik should support a Blender-like instance-based material graph.

Pipeline:

```text
Material Graph Editor
  ↓
Material Graph IR
  ↓
Generated WGSL surface function
  ↓
Kinetik Standard PBR Renderer
```

Users design surfaces. The renderer owns lighting, render passes, bind groups, shadows, and platform compatibility.

### 11.6 Surface Output

Initial graph output:

```text
PBR Surface Output
  Base Color
  Metallic
  Roughness
  Normal
  Emissive
  Alpha
  Ambient Occlusion
```

### 11.7 Initial Shader Graph Elements

Inputs:

- UV.
- Texture Coordinate.
- Vertex Color.
- Object Position.
- World Position.
- Normal.
- Time.
- Camera Vector.

Constants:

- Float.
- Vec2.
- Vec3.
- Vec4.
- Color.

Texture:

- Image Texture.
- Normal Map.

Math:

- Add, subtract, multiply, divide.
- Power, clamp, min, max.
- Sine, cosine.
- Smoothstep.
- Lerp/mix.

Vector:

- Dot.
- Cross.
- Normalize.
- Length.
- Split/combine.

Color:

- ColorRamp.
- Mix Color.
- RGB/HSV later.

Procedural:

- Noise.
- Checker.
- Gradient.
- Voronoi later.

Utility:

- Fresnel.
- Remap.
- One Minus.
- Step/Compare.

### 11.8 Custom Shaders

Support in levels:

1. **Surface shader customization** generated from graph or user WGSL surface functions.
2. **Full pass shaders** for advanced custom drawing.
3. **Render graph plugins** later.

Do not begin with arbitrary user-owned wgpu pipelines.

### 11.9 Shader Hot Reload

Behavior:

1. Detect shader/graph change.
2. Compile generated WGSL.
3. If success, rebuild affected pipelines and swap.
4. If failure, keep previous valid shader and show editor error.
5. Never black-screen editor due to shader typo.

### 11.10 Renderer Roadmap

```text
Foundation → Unlit → Basic PBR → Shadows/IBL → Shader Graph → Render Graph → Advanced
```

---

## 12. Physics System

### 12.1 Philosophy

> Physics authoring is instance-based. Physics simulation is server-owned. Scripts talk through safe high-level APIs, never directly to Rapier.

### 12.2 Core Concepts

- RigidBody: object participating in simulation.
- Collider: shape used for collision.
- Area/Trigger: overlap detection without physical blocking.
- CharacterBody: gameplay-focused movement controller.

### 12.3 Physics Instances

3D:

```text
StaticBody3D
RigidBody3D
CharacterBody3D
Area3D

BoxCollider3D
SphereCollider3D
CapsuleCollider3D
CylinderCollider3D
ConvexCollider3D
MeshCollider3D
HeightfieldCollider3D

FixedJoint3D
HingeJoint3D
SliderJoint3D
BallJoint3D

RayCast3D
ShapeCast3D
```

2D later follows parallel naming.

### 12.4 Ownership Rules

1. A collider belongs to the nearest ancestor physics body.
2. Collider without physics ancestor is invalid at runtime; editor offers quick fix.
3. Physics body controls transform of visual descendants.
4. Dynamic bodies nested under dynamic bodies require warnings; use joints for physical attachment.

Example:

```text
Crate
  RigidBody3D
    BoxCollider3D
    MeshRenderer3D
```

### 12.5 Body Properties

```text
mode: Static | Dynamic | Kinematic
mass
gravity_scale
linear_velocity
angular_velocity
linear_damping
angular_damping
can_sleep
continuous_collision_detection
axis locks
collision_layer
collision_mask
```

### 12.6 Collider Properties

```text
shape
is_trigger
friction
restitution
density
collision_layer
collision_mask
offset_transform
```

### 12.7 Update Order

```text
1. Process input/events
2. Run Update(dt)
3. Apply queued transform/physics commands
4. Run fixed physics step
5. Sync physics results to scene transforms
6. Emit physics signals/events
7. Run physics callbacks if configured
8. Render
```

### 12.8 Luau Physics API

```lua
body:apply_force(Vec3.new(0, 10, 0))
body:apply_impulse(Vec3.new(0, 8, 0))
body:set_linear_velocity(Vec3.new(1, 0, 0))
body:wake_up()

local hit = physics.raycast({
    origin = camera.position,
    direction = camera.forward,
    max_distance = 100,
    mask = "World"
})
```

### 12.9 Character Controller

Kinetik should provide a friendly `CharacterBody3D`.

```lua
function PhysicsUpdate(dt: number)
    local move = input.get_vector("left", "right", "forward", "back")
    self.character:move_and_slide(Vec3.new(move.x, 0, move.y) * 6.0)
end
```

### 12.10 Collision Layers

Expose named layer/mask checkboxes in the editor. Store as bitmasks internally.

---

## 13. Terrain, Default World, Time, Sky, and Lighting

### 13.1 Default New 3D Project

New projects should start alive, not empty.

```text
Game
  Workspace
    Terrain optional by template
    Baseplate
      StaticBody3D
      BoxCollider3D
      MeshRenderer3D
    Camera
      Camera3D

  Lighting
    Sun
      DirectionalLight3D
    Sky
    Atmosphere

  Physics
  Audio
```

### 13.2 Templates

- Empty 3D: camera + lighting.
- Baseplate: floor + camera + sun + sky.
- Terrain: terrain + camera + sun + sky.
- First Person: baseplate + player controller + camera.
- Third Person: character body + camera rig.

### 13.3 Terrain

Terrain appears as a Workspace object:

```text
Workspace
  Terrain
    Terrain3D
```

Internally managed by terrain systems.

#### Heightmap Terrain

Good for:

- Landscapes.
- Sculpt/paint tools.
- Rapier heightfields.
- Efficient outdoor scenes.

#### Voxel Terrain

Long-term flagship possibility, Roblox-inspired:

- Caves.
- Destruction.
- Procedural worlds.
- Material voxels.

Recommendation:

> Start with heightmap terrain as the first implementation, but architect `kinetik-terrain` so voxel terrain can exist later.

### 13.4 Terrain API

Heightmap:

```lua
local terrain = game.Workspace.Terrain
terrain:set_height(x, z, height)
terrain:paint_material(x, z, "Grass")
terrain:get_height(x, z)
```

Voxel later:

```lua
terrain:fill_region(region, "Rock")
terrain:carve_sphere(position, radius)
terrain:set_voxel(x, y, z, "Sand")
```

### 13.5 Time of Day

Kinetik should have Roblox-like time-of-day controlled by Lighting.

```lua
game.Lighting.time_of_day = 18.25
```

Lighting properties:

```text
time_of_day: 0.0..24.0
day_length_seconds
sun_mode: TimeOfDay | ManualDirection
sky_mode: Procedural | Skybox | HDRI
ambient_color
fog_color
fog_density
exposure
```

Sun behavior:

```text
Sun.follow_time_of_day = true
```

If disabled, sun direction is manual.

Principle:

> Every 3D scene has a world environment. Camera, lighting, sky, physics, and terrain are first-class objects/services, not hidden engine state.

---

## 14. Scripting: Luau Bridge and API

### 14.1 Decision

Kinetik uses **Luau** as the intended first-class scripting language.

Rationale:

- Familiar to Roblox creators.
- Optional static typing.
- Better autocomplete and editor tooling.
- More scalable for large games.
- Stronger fit for creator platform identity.

Plain Lua via `mlua` would be easier, but Luau is the better product decision.

### 14.2 Bridge Architecture

Crates:

```text
kinetik-script
kinetik-script-luau
```

`kinetik-script` defines runtime-agnostic concepts:

- Script lifecycle.
- Script handles.
- Script errors.
- Script component model.
- API registration contracts.

`kinetik-script-luau` owns:

- Luau VM integration.
- Safe Rust wrappers.
- Userdata/handle bindings.
- Type definitions generation.
- Sandboxing.
- Hot reload.

### 14.3 Safety Rules

Luau may hold:

- Instance handles.
- Resource handles.
- Component handles.
- Safe userdata wrappers.

Luau must not hold:

- Raw Rust pointers.
- Direct borrowed references.
- Rapier handles.
- wgpu internals.
- OS capabilities.
- Arbitrary file/network access unless explicitly permitted.

### 14.4 Root Namespaces

Initial namespaces:

```lua
game      -- hierarchy/services
workspace -- convenience alias for game.Workspace
prefabs   -- convenience alias for game.Prefabs
assets    -- resource/addressable loading
input     -- actions/input state
physics   -- queries/global physics
audio     -- playback/buses
ui        -- UI access/helpers
time      -- frame/fixed timing
tasks     -- coroutines/scheduling
debug     -- logs/debug drawing
mathf     -- math helpers
```

Core types:

```lua
Vec2
Vec3
Vec4
Quat
Color
Transform
Rect
Aabb
```

Reserved/later:

```lua
bundles
editor
network
```

`network` is optional/later. Kinetik must not imply hosted server architecture by default.

### 14.5 Game Hierarchy API

```lua
game.Workspace
game.Prefabs
game.Scripts
game.UI
game.Lighting
game.Audio
game.Physics
game.Assets
game.Packages
```

Methods:

```lua
game:get_instance(path)
game:find_node(path)
game:load_scene(path)
game:quit()
game:is_editor()
game:is_playing()
```

### 14.6 Instance API

```lua
instance:name()
instance:set_name(name)
instance:parent()
instance:children()
instance:add_child(child)
instance:remove_child(child)
instance:clone()
instance:queue_free()
instance:get_instance(path)
instance:find_child(name)
instance:has_tag(tag)
instance:add_tag(tag)
instance:remove_tag(tag)
instance:emit_signal(name, ...)
instance:connect(signal_name, callback)
```

Transform:

```lua
instance.position = Vec3.new(0, 3, 0)
instance.rotation = Quat.from_euler(0, 1.57, 0)
instance.scale = Vec3.new(1, 1, 1)
instance:translate(Vec3.new(1, 0, 0))
instance:rotate_y(dt)
instance:look_at(target)
```

### 14.7 Lifecycle

```lua
function Ready()
end

function Update(dt: number)
end

function PhysicsUpdate(dt: number)
end

function Exit()
end
```

Events/callbacks:

```lua
function on_input(event)
end

function on_body_entered(body)
end

function on_area_entered(area)
end

function on_collision_started(event)
end
```

Hot reload state:

```lua
function save_state()
    return { hp = self.hp }
end

function load_state(state)
    self.hp = state.hp or 100
end
```

### 14.8 Input API

Action-first:

```lua
input.is_action_pressed("jump")
input.is_action_just_pressed("jump")
input.is_action_just_released("jump")
input.get_axis("left", "right")
input.get_vector("left", "right", "forward", "back")
input.mouse_position()
input.mouse_delta()
input.set_mouse_mode("captured")
```

### 14.9 Tasks API

Roblox-like scheduling, engine-owned:

```lua
tasks.spawn(fn)
tasks.defer(fn)
tasks.delay(1.0, fn)
tasks.wait(0.5)
tasks.cancel(handle)
```

### 14.10 Debug API

```lua
debug.log(...)
debug.warn(...)
debug.error(...)
debug.draw_line(a, b, Color.red)
debug.draw_ray(origin, direction, Color.green)
debug.draw_sphere(pos, radius, Color.blue)
```

### 14.11 Typed API Definitions

Kinetik should generate Luau type definitions for engine APIs so Studio can provide autocomplete and type checking.

---

## 15. Signal System

### 15.1 Purpose

Decouple systems and gameplay logic.

Luau scripts and Rust systems can emit/listen without hardcoded dependencies.

### 15.2 Requirements

- Typed signal IDs internally.
- String/named signals in authoring API.
- Deterministic dispatch order.
- Safe subscribe/unsubscribe.
- Safe emit during dispatch, queued if necessary.
- Scene/instance lifecycle cleanup.
- Luau connection handles invalidated safely.

### 15.3 API

```lua
local connection = instance:connect("picked_up", function(player)
    debug.log("Picked up by", player:name())
end)

instance:emit_signal("picked_up", player)
connection:disconnect()
```

---

## 16. Editor / Kinetik Studio

### 16.1 Identity

Kinetik Studio should aim for a beautiful, fast, coherent editor. Long-term UI is Vello-powered, custom-rendered, and Blender-grade.

### 16.2 Editor Principles

- Editor manipulates runtime scene model but keeps editor-only state separate.
- Undo/redo is mandatory for destructive or transform edits.
- Defaults should feel alive.
- Inspector should be clear and typed.
- Warnings should teach, not annoy.
- No hidden magic where explicit objects/services would be clearer.

### 16.3 Core Panels

- Explorer/hierarchy.
- Viewport.
- Inspector.
- Asset Browser.
- Shader Graph.
- Console.
- Output/errors.
- Profiler later.
- Package/bundle manager later.

### 16.4 Inspector

Rust reflection/macros generate metadata.

Example:

```rust
#[derive(Inspectable)]
struct Transform3D {
    #[inspect(label = "Position")]
    position: Vec3,
}
```

Inspector must support:

- Sliders.
- Text fields.
- Color pickers.
- Resource pickers.
- Instance references.
- Layer masks.
- Enum dropdowns.
- Warnings.

### 16.5 Gizmos

Viewport gizmos:

- Translate.
- Rotate.
- Scale.
- Collider editing.
- Terrain brushes.
- Light/camera icons.

### 16.6 Shader Graph Editor

Must support:

- Instance search.
- Drag connections.
- Type validation.
- Preview thumbnails.
- Error messages.
- Undo/redo.
- Copy/paste.
- Groups/comments later.

---

## 17. Testing and Quality Strategy

### 17.1 Required Checks

Early CI:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

### 17.2 Test Categories

- Unit tests.
- Serialization roundtrip tests.
- Golden scene fixtures.
- Invalid handle tests.
- Determinism tests.
- Integration vertical slices.
- Shader graph codegen tests.
- Physics event tests.
- Resource import tests.
- Bundle verify/load tests.

### 17.3 Vertical Slice Tests

First golden path:

```text
Create project
Open default scene
Add cube
Attach Luau script
Press play
Move cube/player
Emit signal
Render frame
Save scene
Reload scene
Verify state
```

### 17.4 Definition of Done

A task is done only when:

- Code compiles.
- Tests pass.
- No unrelated files changed.
- Public APIs documented.
- Errors meaningful.
- Assumptions recorded.
- Docs updated if behavior changed.
- Integration points clear.
- No untracked TODOs pretending to be complete.

For editor work:

- Screenshot or visual description.
- Interaction behavior documented.
- Undo/redo considered.

For runtime work:

- Invalid handles tested.
- Failure states tested.
- Cleanup considered.
- Determinism considered.

---

## 18. Scaffolding Order

Because this is a marathon, scaffolding starts with the project nervous system.

### 18.1 First Batch

```text
Repo skeleton
AGENTS.md
Constitution
Architecture overview
Crate map
ADR folder
Task template
Review checklist
CI
Core primitive crates
```

Crates:

```text
kinetik-core
kinetik-scene
kinetik-signal
kinetik-resource
kinetik-test
```

### 18.2 Foundational Implementation Order

```text
Constitution
→ Workspace
→ CI
→ Core IDs/errors/math
→ Scene tree
→ Signal bus
→ Resource manager
→ Serialization
→ App loop
→ Luau bridge skeleton
→ Physics skeleton
→ Renderer skeleton
→ Default baseplate scene
→ Editor shell
→ Shader graph
→ Bundles
→ Terrain
```

This is not cutting the dream. It is building the spine first.

---

## 19. Initial ADRs to Write

```text
0001-core-stack-rust-luau-wgpu-vello.md
0002-instance-scene-model.md
0003-handle-id-system.md
0004-project-organization.md
0005-resource-import-cache-and-metadata.md
0006-standard-renderer-and-shader-graph.md
0007-physics-authoring-model.md
0008-luau-scripting-api.md
0009-bundle-system.md
0010-terrain-and-world-environment.md
```

Each ADR must include:

- Context.
- Decision.
- Consequences.
- Alternatives considered.
- Reopen conditions.

---

## 20. Immediate Agent Assignments

### 20.1 Architect Agent

Mission:

- Create docs, constitution, crate map, ADR stubs.

Do not touch:

- Runtime implementation beyond empty crate scaffolds.

### 20.2 Core Agent

Mission:

- Implement typed IDs, errors, result aliases, basic math reexports/wrappers.

Scope:

- `kinetik-core` only.

### 20.3 Scene Agent

Mission:

- Implement basic instance hierarchy, parent/child operations, stable GUIDs, runtime InstanceId mapping.

Scope:

- `kinetik-scene`.

### 20.4 Signal Agent

Mission:

- Implement deterministic signal bus.

Scope:

- `kinetik-signal`.

### 20.5 Resource Agent

Mission:

- Implement resource handles, source path resolution, metadata sidecar model, in-memory cache skeleton.

Scope:

- `kinetik-resource`.

### 20.6 QA Agent

Mission:

- Attack scene tree, invalid handles, serialization determinism, signal ordering.

### 20.7 Integration Agent

Mission:

- Build `examples/hello_scene` once scene/resource/signal basics exist.

---

## 21. Major Risks and Controls

### 21.1 Architectural Entropy

Risk: many agents produce many incompatible mini-engines.

Controls:

- Constitution.
- ADRs.
- Strict task scope.
- Architect review.
- Crate boundaries.
- CI and integration tests.

### 21.2 Editor Overgrowth

Risk: editor becomes monolithic and absorbs runtime logic.

Controls:

- Runtime/editor dependency boundary.
- Editor-only state separated.
- Inspector/reflection APIs defined carefully.

### 21.3 Luau Bridge Complexity

Risk: Luau integration becomes harder than expected.

Controls:

- Isolate in `kinetik-script-luau`.
- Keep script API handle-based.
- Generate type definitions.
- Avoid exposing internals.

### 21.4 Shader Graph Complexity

Risk: graph becomes arbitrary shader chaos.

Controls:

- Surface-output model first.
- Engine-owned renderer.
- Validated Material Graph IR.
- Generated WGSL.
- Safe fallback materials.

### 21.5 Asset/Bundle Security

Risk: runtime remote bundles inject unsafe scripts/content.

Controls:

- Bundle permission classes.
- Hash/signature verification.
- Script sandboxing.
- Trusted source policy.

### 21.6 Serialization Drift

Risk: scene/resource formats change casually.

Controls:

- Versioned formats.
- Migrations.
- Golden fixtures.
- Explicit approval for format changes.

---

## 22. Final North Star

Kinetik succeeds if creators can:

1. Open a new project and immediately see a beautiful default world.
2. Organize their game clearly in Workspace, Prefabs, Scripts, UI, Lighting, Audio, Physics, Assets, and Packages.
3. Add primitives and imported GLB models naturally.
4. Build clone-ready prefabs.
5. Script gameplay in typed Luau with friendly APIs.
6. Use physics through intuitive instances.
7. Create expressive procedural materials through a shader graph.
8. Commit the project cleanly to Git.
9. Build asset bundles for cloud/CDN/modular runtime loading.
10. Scale the project without the engine architecture collapsing.

The marathon principle:

> We are not minimizing ambition. We are maximizing coherence.

Kinetik should feel like one thoughtful engine, even if a hundred agents helped build it.
