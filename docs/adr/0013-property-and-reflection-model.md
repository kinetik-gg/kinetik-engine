# ADR 0013: Property and Reflection Model

## Status

Accepted as initial project direction.

## Context

Kinetik needs one shared source of truth for editable, serializable, scriptable,
and automatable instance state.

The editor inspector, MCP commands, scene serialization, prefab overrides, Luau
bindings, validation, undo/redo, and diagnostics must agree on which properties
exist and how they may be changed.

## Decision

Kinetik uses class-level reflected property descriptors as the single source of
truth for editable, serializable, scriptable instance state.

Instances store property values. Instance classes define the valid property
schema.

Reflection metadata is runtime-owned. Editor UI, MCP commands, scene/prefab
serialization, diagnostics, and Luau binding generation consume it without
making separate property rules.

Canonical reflected property paths use PascalCase:

```text
Name
Transform.Position
Transform.Rotation
Transform.Scale
Visible
Mesh
Material
```

Luau may expose idiomatic aliases later, but reflection and serialization use
canonical property paths.

## Property Descriptor Shape

A reflected property descriptor should define at least:

- Canonical property path.
- Display name.
- Value type.
- Default value.
- Serialization key.
- Whether it is serialized.
- Whether it is editor-editable.
- Whether it is scriptable.
- Whether it is mutable during play mode.
- Read-only reason when locked.
- Editor hint.
- Validation rules.
- Documentation/help text.

Editor hints should cover common inspector and MCP needs:

- Free number.
- Slider.
- Angle.
- Color picker.
- Asset picker.
- Instance reference picker.
- Enum/dropdown.
- Checkbox.
- Advanced/collapsed display.
- Runtime-only display.

## Consequences

- The editor inspector and MCP property editing share validation behavior.
- Scene and prefab serialization can reject unknown or invalid properties.
- Prefab overrides can reference stable canonical property paths.
- Luau bindings can be generated from the same schema used by the editor.
- Diagnostics can point to exact properties with stable paths.
- Runtime crates may expose reflection metadata, but editor-specific UI code
  stays out of runtime crates.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Separate property definitions for editor, scripting, serialization, and MCP.
  - Rejected because the systems would drift and create inconsistent behavior.
- Dynamic per-instance arbitrary property bags as the primary model.
  - Rejected for core engine classes because validation, tooling, and generated
    bindings need stable class-level schemas.
- Editor-owned reflection metadata.
  - Rejected because runtime serialization, scripting, diagnostics, and tests
    also need the same property truth.

## Reopen Conditions

- Class-level reflection cannot express required engine/editor workflows.
- Prefab override or serialization requirements need a different property path
  model.
- Luau binding generation exposes serious mismatch with canonical property
  descriptors.
- A better reflection architecture emerges with clear migration path.
