# ADR Index

Use this index before opening individual ADRs. It is a discovery aid, not a
replacement for the ADR text.

Agents should:

1. Match the task area and keywords.
2. Read the ADRs listed in "read before touching".
3. Search ADR text only when the index does not clearly cover the task.
4. Update this index whenever an ADR is added, superseded, or materially
   amended.

## Index

| ADR | Status | Area | Keywords | Read Before Touching |
| --- | --- | --- | --- | --- |
| [0001: Core Stack](0001-core-stack-rust-luau-wgpu-vello.md) | Accepted | Stack, editor, renderer, scripting | Rust, Luau, wgpu, Vello | Engine stack, editor UI technology, renderer/scripting direction |
| [0002: Instance Scene Model](0002-instance-scene-model.md) | Accepted | Scene | Game root, services, hierarchy, instances | Scene tree, instance hierarchy, default scene |
| [0003: Handle ID System](0003-handle-id-system.md) | Accepted | Core identity | handles, IDs, GUIDs, runtime IDs | Typed IDs, stable GUIDs, runtime/edit identity |
| [0004: Project Organization](0004-project-organization.md) | Accepted | Project layout | Kinetik.toml, assets, generated output | Project scaffolding, source/generated paths |
| [0005: Resources and Metadata](0005-resource-import-cache-and-metadata.md) | Accepted | Resources | manifest, import cache, metadata, asset GUID | Asset identity, import cache, resource diagnostics |
| [0006: Renderer and Shader Graph](0006-standard-renderer-and-shader-graph.md) | Accepted | Rendering | wgpu, PBR, shader graph, WGSL | Renderer, materials, shader graph, render diagnostics |
| [0007: Physics Authoring](0007-physics-authoring-model.md) | Accepted | Physics | rigid body, collider, character controller | Physics authoring, Rapier boundary |
| [0008: Luau Scripting API](0008-luau-scripting-api.md) | Accepted | Scripting | Luau, lifecycle, sandbox, APIs | Script lifecycle, script API exposure |
| [0009: Bundle System](0009-bundle-system.md) | Accepted | Bundles | packaging, export, build output | Bundle/export pipeline |
| [0010: Terrain and World Environment](0010-terrain-and-world-environment.md) | Accepted | Terrain/world | terrain, world, environment | Terrain or world-environment features |
| [0011: Agentic Editor MCP Contract](0011-agentic-editor-mcp.md) | Accepted | MCP/editor automation | MCP, agents, tools, editor commands | MCP surfaces, semantic automation |
| [0012: Project Serialization Format](0012-project-serialization-format.md) | Accepted | Serialization | TOML, RON, deterministic, Git-friendly | Project, manifest, scene, prefab file formats |
| [0013: Property and Reflection Model](0013-property-and-reflection-model.md) | Accepted | Reflection/properties | descriptors, property values, validation | Reflected properties, inspector/script bindings |
| [0014: Editor Command and Semantic Change Model](0014-editor-command-and-semantic-change-model.md) | Accepted | Commands/editor | command, change record, undo, dirty state | Mutations, undo/redo, command validation |
| [0015: Diagnostics Model](0015-diagnostics-model.md) | Accepted | Diagnostics | diagnostic code, severity, repairability | Diagnostics, errors, repair workflows |
| [0016: Prefab and Package Override Model](0016-prefab-and-package-override-model.md) | Accepted | Prefabs/packages | prefab, package, override, apply | Prefab/package source and override behavior |
| [0017: Unsafe Rust Boundary Policy](0017-unsafe-rust-boundary-policy.md) | Accepted | Safety | unsafe, FFI, boundary, wrapper | Any unsafe/FFI work |
| [0018: Dependency Governance](0018-dependency-governance.md) | Accepted | Dependencies | crate, license, transitive, boundary | Any dependency addition, upgrade, or exposed dependency type |
| [0019: Edit, Play, and Runtime State Boundaries](0019-edit-play-and-runtime-state-boundaries.md) | Accepted | Runtime/editor boundary | edit world, play world, runtime sandbox | Play mode, runtime mutation, edit/runtime sync |
| [0020: Runtime Execution Model](0020-runtime-execution-model.md) | Accepted | Runtime | lifecycle, single-threaded, scripting | Runtime execution, script lifecycle |
| [0021: Permissioned HTTP Service](0021-permissioned-http-service.md) | Accepted | Permissions/networking | HTTP, permission, provenance, diagnostics | Any script/network/HTTP permission surface |
| [0022: Runtime Frame and Event Order](0022-runtime-frame-and-event-order.md) | Accepted | Runtime frame/events | frame order, fixed step, signals, snapshot | Frame scheduler, events, structural sync points |

## Common Task Areas

- Scene/instance hierarchy: ADR 0002, 0003, 0013.
- Project layout and source files: ADR 0004, 0012.
- Resources/assets/import cache: ADR 0005, 0012, 0018.
- Rendering/materials/shaders: ADR 0001, 0006, 0018.
- Physics: ADR 0007, 0018, 0022.
- Luau/scripting: ADR 0008, 0017, 0018, 0020, 0021, 0022.
- Commands/undo/dirty state: ADR 0014, 0015, 0019.
- Diagnostics: ADR 0015 plus the domain ADR for the source system.
- Runtime/play mode: ADR 0019, 0020, 0022.
- MCP/editor automation: ADR 0011, 0014, 0015, 0019.
- Dependencies: ADR 0018 plus the relevant M4 dependency proposal.
- Unsafe/FFI: ADR 0017 plus an explicit approved exception.

## Index Maintenance

This file must stay current. When an ADR changes status, scope, or keyword
coverage, update the table in the same PR.
