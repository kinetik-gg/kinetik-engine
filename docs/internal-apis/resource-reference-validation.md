# Resource Reference Validation Contract

## Purpose

Define how scene and property values point at project assets so
`ResourceDatabase` can validate references deterministically without guessing at
serialized formats, importer internals, or runtime handles.

## Reference Shape

Durable asset references are `AssetGuid` plus `AssetPath`.

- `AssetGuid` is the stable identity used for rename/move resilience.
- `AssetPath` is the readable `res://` project location used for diffs,
  diagnostics, repair, and human review.
- A reference is valid when the database can resolve the GUID to a manifest entry
  and the stored path either matches that entry or can be reported as stale path
  metadata for repair.
- A GUID/path mismatch is not silently repaired during validation. It produces a
  diagnostic with both the stored reference and the manifest entry.

`ResourceId` is not durable asset identity. It is a typed engine ID suitable for
runtime/import/database-owned handles after a durable asset reference has already
been resolved.

`ResourceHandle` wraps `ResourceId` for loaded or database-derived resources. It
must not be serialized as the source-of-truth scene/property asset reference.

## Property Mapping

Reflected properties that point at assets should use a Kinetik-owned
asset-reference value carrying:

- `AssetGuid`.
- `AssetPath`.
- Optional expected asset kind or descriptor constraint when the property schema
  needs one, such as material, mesh, prefab, texture, or script.

The exact reflected value and serialized representation are follow-up work. That
implementation must preserve ADR 0005 by storing GUID-backed references with
readable paths and must not expose importer/parser types.

Existing `PropertyValue::ResourceId` represents a derived resource handle value,
not the durable scene/source reference contract for asset properties. Future
implementation may keep it for runtime-only values, replace it for serialized
asset references, or add a separate reflected asset-reference value through a
focused public API and serialized-format review.

## Validation Behavior

`ResourceDatabase` reference validation should be deterministic and IO-free:

1. Validate the `AssetPath` syntax using the `res://` contract.
2. Resolve the `AssetGuid` in the committed manifest.
3. Compare the stored `AssetPath` with the manifest path for that GUID.
4. Optionally compare the resolved entry against descriptor-level asset kind
   constraints once asset kinds exist.
5. Report diagnostics in stable scene traversal and property-path order when
   validating scene/property values.

Validation does not load source files, run importers, inspect generated cache
artifacts, or assign replacement GUIDs.

## Diagnostics

Invalid asset-reference diagnostics include the most specific stable target data
available:

- Scene instance GUID and scene path.
- Reflected property path.
- Stored asset GUID.
- Stored `res://` path, when present and syntactically valid enough to report.
- Manifest asset path for the same GUID, when available.
- Diagnostic source `ResourceDatabase`.
- Blocking scope for the workflow being validated, such as `Edit`, `Save`,
  `Play`, `Build`, or `Bundle`.

Suggested diagnostic cases:

- Missing GUID in the manifest.
- Malformed `res://` path.
- GUID/path mismatch.
- Missing readable path beside a GUID-backed reference.
- Asset kind mismatch once kind constraints exist.

Diagnostics may suggest repair only when the repair is mechanical and safe, such
as updating a stale readable path to the manifest path for the same GUID.

## Acceptance Criteria For Implementation

- Add a Kinetik-owned reflected asset-reference value or equivalent approved
  public type instead of using parser/importer types.
- Validate scene/property asset references through `ResourceDatabase` using
  deterministic traversal order.
- Preserve GUID-backed identity and readable `res://` paths.
- Treat `ResourceId`/`ResourceHandle` as derived handles, not durable serialized
  identity.
- Emit structured diagnostics with instance, property, GUID, and path context.
- Add focused unit tests and golden fixtures if serialized scene/property output
  changes.
