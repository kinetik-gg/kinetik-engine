# ResourceDatabase Contract

## Purpose

Define the engine-owned resource database over committed manifests, source
assets, generated import cache metadata, and resource reference validation.

## Owning Crates

- `kinetik-resource`: asset identity, `res://` paths, manifest model, project
  layout validation, resource database, import cache metadata, and resource
  diagnostics.
- Format importer modules: parse source files and produce Kinetik-owned import
  artifacts after dependency approval.
- `kinetik-render`, `kinetik-physics`, `kinetik-script`, and `kinetik-bundle`:
  consume validated resource references and imported outputs.

## ResourceDatabase Contract

`ResourceDatabase` is built from committed project manifests and observable
project source state.

Required capabilities:

- Lookup by asset GUID.
- Lookup by `res://` path.
- Detect missing, moved, duplicate, and invalid assets.
- Validate resource references from scene/property values.
- Report importer ID/version/settings hash and cache schema metadata.
- Distinguish source assets from generated import outputs.
- Provide deterministic iteration for tests, UI, MCP, and bundles.

The import cache is disposable and can be rebuilt from source assets and
manifests.

## Dependency Boundaries

- Importer dependencies follow `docs/dependency-proposals/asset-import.md`.
- Serialization dependencies follow
  `docs/dependency-proposals/serialization-toml-ron.md`.
- Third-party parser/decoder structs must be converted to Kinetik-owned import
  artifacts before crossing boundaries.

## Serialized-Format Impact

This spec does not change manifest formats.

Future manifest, import settings, cache records, or resource reference formats
require focused serialized-format issues and golden fixtures.

## Diagnostics Behavior

Resource diagnostics report:

- Missing source asset.
- Duplicate GUID or path.
- Moved source with ambiguous identity.
- Invalid `res://` path.
- Missing import output that can be rebuilt.
- Importer version/settings mismatch.
- Invalid scene/property resource reference.

Diagnostics include asset GUIDs and paths when available.

## Public API Constraints

- Public APIs expose Kinetik-owned asset references, resource IDs, import
  records, and diagnostics.
- Runtime users do not receive source parser or file-watcher types.
- Render/physics/script systems should not own asset identity repair.

## Follow-Up Issues

- M11 resource database scaffold.
- M11 reference validation.
- M11 missing/duplicate diagnostics.
- M28 texture/glTF import after approved dependency installation.
