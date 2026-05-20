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

Import cache metadata held by `ResourceDatabase` is an index of
`ImportCacheRecord` values keyed by asset GUID. It reports source content hash,
importer ID/version, import settings hash, and cache schema version without
loading generated outputs or invoking importers. Cache records are ordered
deterministically by asset GUID and remain separate from committed manifest
identity: a cache record does not create, repair, or replace a manifest entry.

## Asset Dependency Lookup Contract

Asset dependency lookup is a database view over Kinetik-owned import artifact
metadata. It must not expose parser/importer structs, file-watcher events, or
generated cache file paths as public engine data.

Future import artifacts should report dependency edges as durable asset
references:

- Dependent asset GUID.
- Dependency asset GUID.
- Dependency `res://` path retained for readability and repair.
- Optional dependency role, such as material texture, mesh material, prefab child
  asset, script module, or bundle include.
- Optional source range or importer-owned context when it can be reported through
  Kinetik diagnostics without exposing parser types.

`ResourceDatabase` should support both lookup directions:

- Dependencies of an asset: all assets directly required by a given asset GUID.
- Dependents of an asset: all assets that directly require a given asset GUID.

Iteration order is deterministic:

1. Dependent asset GUID ascending.
2. Dependency asset GUID ascending.
3. Dependency `res://` path ascending.
4. Dependency role text ascending when present.

Dependency lookup is read-only. It does not import assets, load generated cache
outputs, rewrite manifests, or repair stale references. Missing dependency
targets, GUID/path mismatches, and stale dependency metadata produce structured
diagnostics with the dependent asset GUID/path and dependency GUID/path when
available.

Implementation acceptance criteria:

- Add Kinetik-owned dependency edge metadata instead of parser/importer types.
- Provide deterministic dependencies-of and dependents-of lookup APIs.
- Preserve the separation between committed manifest identity, import cache
  metadata, and generated outputs.
- Emit diagnostics for missing or stale dependency targets without assigning
  replacement GUIDs.
- Add focused tests for ordering, both lookup directions, and diagnostics.

## Resource Reference Mapping

Scene and property asset references use the mapping defined in
`resource-reference-validation.md`.

Durable references are `AssetGuid` plus `AssetPath`. The GUID is stable asset
identity; the `res://` path is readable location and repair context.
`ResourceId` and `ResourceHandle` are derived runtime/import/database handles
after a durable reference resolves. They are not the source-of-truth identity for
serialized scene/property asset references.

Reference validation should resolve GUIDs through the committed manifest, compare
stored paths with manifest paths, and emit structured diagnostics instead of
silently replacing missing identity or stale paths.

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
- GUID/path mismatch in a scene/property asset reference.

Diagnostics include asset GUIDs, paths, scene instance identity, and reflected
property paths when available.

## Public API Constraints

- Public APIs expose Kinetik-owned asset references, resource IDs, import
  records, and diagnostics.
- Runtime users do not receive source parser or file-watcher types.
- Render/physics/script systems should not own asset identity repair.

## Follow-Up Issues

- M11 resource database scaffold.
- M11 reference validation.
- M11 missing/duplicate diagnostics.
- Reflected asset-reference value shape and serialized-format review.
- M28 texture/glTF import after approved dependency installation.
