# Scene and Reflection Contract

## Purpose

Define how scene instances and reflected properties remain the shared source of
truth for editor, serialization, scripting, MCP, diagnostics, and tests.

## Owning Crates

- `kinetik-scene`: instance hierarchy, scene documents, class registry,
  structural mutations, stable scene paths, edit-mode IDs, GUIDs, and property
  storage on instances.
- `kinetik-reflect`: property descriptors, value containers, validation rules,
  editor/script/serialization policy flags, and editor hints.
- `kinetik-core`: typed IDs, math/value primitives, and shared diagnostics
  primitives.

Editor, MCP, serialization, scripting, and runtime systems consume these
contracts. They do not define separate property or hierarchy rules.

## Scene Contract

Scene state is a deterministic instance hierarchy with one root `Game`
instance. Default services under `Game` are real serialized instances.

Scene APIs must support:

- Lookup by runtime/edit instance ID, stable GUID, and scene path.
- Deterministic traversal and child order.
- Structural mutations validated before application.
- Reflected property reads and writes validated against class descriptors.
- Document conversion without parser-specific types in public contracts.

Structural changes during runtime iteration must be queued for safe sync points
by runtime systems; edit-mode commands may apply immediately after validation.

## Reflection Contract

Reflection descriptors are class-level metadata. They define:

- Canonical PascalCase property paths.
- Value type and default value.
- Serialization key and serialized flag.
- Editor-editable, scriptable, and play-mode mutability flags.
- Read-only reasons, editor hints, validation rules, and help text.

Instances store property values. Descriptors define which values are legal.

## Dependency Boundaries

- Scene serialization follows
  `docs/dependency-proposals/serialization-toml-ron.md`.
- Luau binding generation follows `docs/dependency-proposals/luau.md`.
- Reflection and scene public APIs expose Kinetik-owned descriptors and values,
  not serializer, VM, UI, or MCP types.

## Serialized-Format Impact

This spec does not change source formats.

Scene/prefab serialization changes require focused issues, deterministic output,
unknown-property diagnostics, and golden fixtures.

## Diagnostics Behavior

Scene and reflection validation report diagnostics for:

- Missing root or duplicate roots.
- Duplicate GUIDs.
- Unknown classes.
- Invalid scene paths or ambiguous names.
- Unknown reflected properties.
- Type mismatches.
- Read-only or play-mode-locked writes.
- Descriptor validation failures.

Diagnostics include stable instance GUIDs, scene paths, class names, and
property paths when available.

## Public API Constraints

- Scene and reflection APIs remain engine-owned and usable without editor/MCP
  crates.
- Editor UI, MCP, and Luau aliases may adapt presentation but must map back to
  canonical descriptors and scene identities.
- Public APIs must not expose parser-specific RON/TOML types.

## Follow-Up Issues

- M7 built-in 3D class descriptor set.
- M7 transform and spatial contracts.
- M13 property command validation.
- M19 inspector descriptor rendering.
