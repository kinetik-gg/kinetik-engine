# Agentic Editor MCP Contract

Kinetik exposes editor and runtime automation through a Model Context Protocol
server owned by the editor process. The MCP surface exists so agents can perform
mechanical game-development work while humans keep authority over gameplay feel,
visual taste, product direction, and architectural changes.

## Mission

Let Codex and other LLM agents inspect, edit, run, and troubleshoot Kinetik
projects through explicit, validated editor operations.

## Scope

The MCP contract covers:

- Project and scene inspection.
- Instance creation, deletion, parenting, duplication, and lookup.
- Property reads and writes through editor-facing reflection metadata.
- Asset import, reimport, lookup, and dependency inspection.
- Script attachment, script diagnostics, and safe script execution controls.
- Play, pause, stop, step, and simulation state inspection.
- Selection, viewport focus, editor panel state, and command history.
- Diagnostics for errors, warnings, missing assets, invalid references, and tests.

The MCP contract does not replace:

- Human creative review.
- Gameplay tuning.
- Art direction.
- Serialized format governance.
- Public API review.
- ADRs for architectural changes.

## Authority Model

Agents operate inside explicit task scopes. Every mutating MCP request must be
attributable to an agent session, an editor command, and an undo group.

The editor remains the source of truth for:

- Project state.
- Scene state.
- Selection state.
- Undo and redo.
- Validation.
- Serialization.
- Asset import state.
- Play mode boundaries.

Agents must not write project files behind the editor when the editor is open.
They must request changes through MCP commands so validation, undo, diagnostics,
and dirty-state tracking stay coherent.

## Stable Object Addressing

Agents need durable, human-readable references without depending on transient
memory addresses. MCP responses should expose multiple identifiers:

- Stable serialized GUID for saved objects.
- Runtime typed ID for the current editor session.
- Scene path for human readability.
- Instance type name.
- Display name.

Requests may address objects by GUID, runtime ID, or scene path. Mutating
operations should prefer GUIDs when available and must report ambiguity instead
of guessing.

## Command Shape

MCP commands should be small, explicit, and undoable.

Examples:

```text
scene.list_instances
scene.get_instance
scene.create_instance
scene.delete_instance
scene.reparent_instance
scene.set_property
scene.get_property
scene.duplicate_instance
asset.import
asset.reimport
asset.list_dependencies
script.attach
script.get_diagnostics
prefab.list_overrides
prefab.apply_override
prefab.revert_override
prefab.update_source
play.start
play.stop
play.step
editor.select
editor.focus_viewport
editor.undo
editor.redo
diagnostics.list
```

Each command returns:

- Success or failure.
- Changed object references.
- Validation diagnostics.
- Undo group ID for mutating commands.
- Human-readable summary.

Failures must be meaningful. A command should explain why it failed and, when
safe, what action would make it valid.

## Property Editing

Properties exposed to agents must come from editor-facing reflection metadata.
An MCP property write must validate:

- Object identity.
- Property existence.
- Property type.
- Value range.
- Read-only status.
- Runtime/editor mode restrictions.
- Serialization impact.

Property writes must not bypass the same validation used by the inspector.

## Play Mode

MCP must distinguish edit mode from play mode.

Edit-mode changes affect the saved project model. Play-mode changes affect the
running simulation unless the editor explicitly supports applying them back to
the project.

Agents must be able to inspect play-mode state, logs, script errors, physics
state, and selected runtime instances. Applying play-mode changes back to edit mode
requires a deliberate editor command and clear diagnostics.

Mutating MCP commands must declare or imply a clear target mode. Ambiguous
commands must fail instead of guessing whether the agent meant saved edit state
or live runtime state.

## Diagnostics

Diagnostics are first-class MCP outputs. Agents should be able to ask:

- What is broken?
- Which object owns the problem?
- Which system reported it?
- Is it blocking play, build, import, or save?
- What changed since the last command?

Diagnostics should use stable codes when possible so agents can recognize and
repair repeated classes of problems.

Diagnostics describe current project or runtime health. Logs describe
chronological events. MCP may expose both, but repair workflows should be driven
by structured diagnostics rather than free-form log text.

Suggested MCP diagnostic commands:

```text
diagnostics.list
diagnostics.get
diagnostics.explain
diagnostics.apply_fix
```

Automated diagnostic fixes must map to editor commands.

## Human-in-the-Loop Boundaries

The MCP server may automate mechanical work, but it must not silently make
judgment-heavy choices. Agents should request human review for:

- Gameplay feel.
- Visual style.
- Animation timing.
- Level composition quality.
- Naming of user-facing concepts when ambiguous.
- Public API changes.
- Serialized format changes.
- Dependency additions.
- Broad refactors.

## Security and Safety

The MCP server should default to project-local authority.

It must not expose arbitrary filesystem, network, shell, or operating-system
access through editor commands. Any future capability that crosses project
boundaries needs a separate ADR and permission model.

## Implementation Direction

The first implementation should be deliberately narrow:

1. Read-only project, scene, selection, and diagnostics inspection.
2. Undoable scene instance creation, deletion, parenting, and property editing.
3. Play, stop, and log/diagnostic inspection.
4. Asset import and script troubleshooting.

The MCP surface should grow only when a real editor workflow needs it.

Mutating MCP tools must map to editor commands from the semantic change model.
They must not bypass command validation, undo grouping, diagnostics, dirty-state
tracking, or structured change records.
