# Kinetik Milestones

This roadmap is the local planning source. GitHub issues are the executable
backlog. Each implementation issue should map to one branch/worktree and one
focused patch.

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

## M10: First Vertical Slice

Goal: prove the smallest end-to-end game-development workflow.

Key outputs:

- Create/scaffold project.
- Load scene.
- Add instance.
- Set reflected property.
- Save and reload.
- Enter play mode.
- Run deterministic frame steps.
- Inspect diagnostics.

Representative issues:

- Hello scene upgrade.
- End-to-end project scaffold/load/save test.
- First play-mode smoke.
- MCP-driven vertical slice smoke.
