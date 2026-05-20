# ADR 0012: Project Serialization Format

## Status

Accepted as initial project direction.

## Context

Kinetik projects must be readable by humans, reviewable in Git, editable by
agents, and safe for Kinetik Studio to scaffold and maintain.

The project workspace decision requires normal source folders, centralized
metadata manifests, and disposable generated output. The next decision is the
source serialization format for project settings, manifests, scenes, and
prefabs.

## Decision

Kinetik source project files are deterministic and text-first.

Use TOML for project configuration and project manifests:

```text
Kinetik.toml
project/assets.knmanifest
project/instances.knmanifest
```

Use RON for scene and prefab instance trees:

```text
scenes/main.knscene
prefabs/enemy.knprefab
```

Kinetik will not invent a custom scene syntax for the initial implementation.
Kinetik will not use YAML for project source files. Binary files may be source
assets, but Kinetik-authored project, manifest, scene, and prefab files remain
text-first.

## Consequences

- Git diffs stay readable for project state, manifests, scenes, and prefabs.
- Agents can inspect and edit closed-editor project files with structured text
  tools.
- Kinetik Studio can scaffold a project without asking users to write these
  files manually.
- Scene and prefab trees can be represented naturally without forcing deeply
  nested TOML tables.
- Adding RON parsing will require an approved dependency when implementation
  begins.
- Serialization order, IDs, and formatting must be deterministic.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- TOML for every project file.
  - Rejected for scene and prefab trees because nested instance hierarchies and
    heterogeneous property values become awkward.
- JSON.
  - Rejected as the primary source format because it is noisier for humans and
    reviews.
- YAML.
  - Rejected because implicit typing and parser differences make it too
    error-prone for source-of-truth project files.
- Custom Kinetik syntax.
  - Rejected for the initial implementation because it would spend complexity
    on language design before the engine model is proven.
- Binary source scene files.
  - Rejected because they weaken Git review, agent editing, and recovery.

## Reopen Conditions

- RON cannot express Kinetik scene and prefab data cleanly.
- Deterministic formatting proves too difficult to maintain.
- A better structured text format emerges with clear migration path.
- Tooling, editor, or asset pipeline requirements invalidate the split.
