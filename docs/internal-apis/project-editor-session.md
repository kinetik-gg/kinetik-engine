# Project and EditorSession Contract

## Purpose

Define the boundary between engine-owned project state and editor-owned session
state before M6 project model and M17 editor session implementation.

## Owning Crates

- `kinetik-resource`: project layout paths, asset manifests, resource identity,
  and resource diagnostics.
- Future project-model owner: `Kinetik.toml` identity/settings, active document
  references, and layout validation. If no existing crate owns this cleanly when
  implementation starts, create a scoped issue before adding code.
- `kinetik-editor`: `EditorSession`, selection, active panels, open project
  lifecycle, command history, dirty-state presentation, and play-mode ownership.
- Runtime crates: consume project/scene/resource state through explicit runtime
  inputs; they do not own editor session state.

## Project Contract

`Project` represents editable source workspace state:

- Workspace root path.
- Project identity and engine compatibility from `Kinetik.toml`.
- Canonical project layout paths.
- Active project settings document.
- Known scene, prefab, script, asset, and project manifest document references.
- Structured diagnostics for layout, settings, manifests, and document health.

`Project` must not store editor-only selection, panels, transient viewport
camera, undo stack, or live play-world state.

## EditorSession Contract

`EditorSession` represents an open Kinetik Studio session:

- Open project handle.
- Active scene/prefab/document selection.
- Editor selection and focus.
- Dirty-state explanations from command/change records and saved snapshots.
- Diagnostics view state derived from project/runtime diagnostics stores.
- Edit/play mode ownership and current play-world handle when present.
- Command history, undo stack, redo stack, and active undo group.

`EditorSession` must not create a private project mutation path. All meaningful
project, scene, resource, and script attachment mutations go through commands.

## Dependency Boundaries

- Project serialization dependencies follow
  `docs/dependency-proposals/serialization-toml-ron.md`.
- Editor window/UI dependencies follow
  `docs/dependency-proposals/editor-window-ui.md`.
- Project contracts expose Kinetik-owned paths, IDs, diagnostics, and document
  handles, not `toml`, `ron`, window, UI, or OS-specific types.

## Serialized-Format Impact

This spec does not change serialized formats.

Later implementation issues that define `Kinetik.toml`, project settings fields,
scene references, or layout persistence must go through serialized-format review
and use golden fixtures.

## Diagnostics Behavior

Project validation reports structured diagnostics for:

- Missing required workspace paths.
- Invalid `Kinetik.toml` identity/settings.
- Missing active scene or referenced documents.
- Invalid or duplicate manifest entries.
- Generated `.kinetik` state that can be rebuilt.

EditorSession may filter and present diagnostics, but diagnostics are produced
by the owning engine subsystem.

## Public API Constraints

- Public APIs expose Kinetik-owned project handles, document handles,
  diagnostics, paths, and IDs.
- Editor-only types must not leak into runtime or resource crates.
- External parser/window/UI types must not leak into public engine APIs.

## Follow-Up Issues

- M6 project model scaffold.
- M6 layout validation diagnostics.
- M17 editor session model.
- M20 save/reload through approved serialization boundaries.
