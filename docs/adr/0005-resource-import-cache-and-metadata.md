# ADR 0005: Resources and Metadata

## Status

Accepted as initial project direction.

## Context

Kinetik is a marathon-scale game engine intended to be built by coordinated
human and AI-agent contributors.

Kinetik needs stable asset identity without Unity-style per-file metadata
sidecars. Asset references must survive renames and moves, generated import
output must be disposable, and missing assets must be diagnosed rather than
silently replaced with new identity.

## Decision

Source assets plus committed centralized manifests, ignored cache.

Asset identity lives in `project/assets.ktmanifest`.

Source assets are referenced by both:

- Stable asset GUID.
- Logical `res://` project path.

The asset GUID is the durable identity. The `res://` path is the human-readable
location and import source. Renames and moves update manifest paths while
preserving GUIDs.

Import output under `.kinetik/` is generated and disposable. Cache keys are based
on:

- Asset GUID.
- Source content hash.
- Importer ID.
- Importer version.
- Import settings hash.
- Engine import-cache schema version.

If a source file is missing, moved outside the editor, duplicated with ambiguous
identity, or has invalid import settings, Kinetik emits structured diagnostics.
It must not silently assign a replacement GUID in a way that breaks existing
references.

Asset references in scenes, prefabs, scripts, manifests, and bundles should
prefer GUID-backed references with `res://` paths retained for readability and
repair.

## Consequences

- Project assets remain Git-friendly without sidecar file noise.
- References can survive asset renames and moves.
- Import cache can be deleted and rebuilt from source files and manifests.
- Asset repair tools and MCP agents can reason from stable GUIDs, paths, and
  diagnostics.
- Importers must be deterministic and versioned.
- This decision shapes crate boundaries, public APIs, editor workflows, tests,
  and agent assignments.
- Reopening requires a follow-up ADR with alternatives and migration implications.

## Alternatives Considered

- Per-file sidecar metadata.
  - Rejected as the default because it creates noisy workspaces and fragile
    file-management workflows.
- Path-only asset references.
  - Rejected because renames and moves would break references too easily.
- GUID-only asset references.
  - Rejected because humans, Git diffs, and repair tools need readable project
    paths.
- Treat import cache as source of truth.
  - Rejected because generated output must be disposable.

## Reopen Conditions

- Technical blocker discovered.
- Better option emerges with clear migration path.
- Product direction changes.
