# Asset Import Dependency Proposal

Status: Approved direction, blocked on MSRV/version installation decision.

Related ADRs and docs:

- ADR 0005: Resources and Metadata
- ADR 0017: Unsafe Rust Boundary Policy
- ADR 0018: Dependency Governance
- Assets and bundles: `docs/architecture/assets-and-bundles.md`
- Crate map: `docs/architecture/crate-map.md`

## Decision Needed

Approve the first asset import dependency direction before texture or glTF/GLB
import implementation starts.

The latest checked `image` line reports Rust 1.88 while the workspace declares
Rust 1.80. A compatible older `image` line exists, but choosing it should be an
explicit pin.

Current crate metadata was checked with `cargo info` on 2026-05-20:

| Crate | Observed latest | License | MSRV reported by crate | Upstream | Proposal outcome |
| --- | ---: | --- | --- | --- | --- |
| `image` | `0.25.10` | MIT OR Apache-2.0 | 1.88.0 | `image-rs/image` | Recommended for textures after MSRV decision. |
| `image` older line | `0.25.6` checked | MIT OR Apache-2.0 | 1.70.0 | `image-rs/image` | Possible temporary pin if Rust 1.80 remains fixed. |
| `gltf` | `1.4.1` | MIT OR Apache-2.0 | 1.61 | `gltf-rs/gltf` | Recommended glTF/GLB parser after import contracts exist. |
| `tobj` | `4.0.3` | MIT | Unknown | `Twinklebear/tobj` | Defer; OBJ is not required for first 3D template acceptance. |
| `blake3` | `1.8.5` | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | Unknown | `BLAKE3-team/BLAKE3` | Candidate content hash, but approve separately from format importers. |

## Recommendation

Approve the direction of using `image` for texture decoding and `gltf` for
glTF/GLB import, but do not install either until import cache contracts and MSRV
direction are approved.

Initial import support should be narrow:

- Texture import for PNG/JPEG and any first-template formats explicitly needed.
- glTF/GLB mesh/material data extraction after resource database and material
  contracts exist.
- Deterministic import records keyed by source hash, importer ID/version,
  settings hash, asset GUID, and cache schema version.

Defer OBJ, broad image format defaults, compressed texture pipelines, and
thumbnail generation until focused issues require them.

## Ownership Boundary

- `kinetik-resource` owns asset identity, manifests, import cache metadata, and
  resource diagnostics.
- Format-specific importer modules own third-party parser/decoder use.
- `kinetik-render` owns GPU resource creation, not source file parsing.
- `kinetik-bundle` consumes validated imported outputs, not importer internals.

Third-party decoded structures must be converted to Kinetik-owned import
artifacts before crossing crate boundaries.

## ADR 0018 Checklist

### Why It Is Needed

ADR 0005 requires deterministic, versioned import output and diagnostics for
missing, moved, duplicated, or invalid source assets. First 3D templates need
texture and mesh import eventually, but dependencies must be reviewed first.

### Alternatives Considered

- Hand-write image or glTF parsers.
  - Rejected because parser correctness would dominate engine work.
- Enable all `image` default formats immediately.
  - Deferred because broad codec support increases transitive risk and may
    include formats not needed for first templates.
- Add OBJ import first.
  - Deferred because glTF/GLB is the more relevant first 3D asset workflow.
- Use path-only source hashing without content hash dependency.
  - Rejected by ADR 0005's cache-key direction.

### License Compatibility

`image` and `gltf` report MIT OR Apache-2.0. `tobj` reports MIT. `blake3`
reports permissive alternatives including CC0 and Apache variants. Installation
PRs must record exact transitive licenses.

### Maintenance Health

`image-rs`, `gltf-rs`, and BLAKE3 are established. The latest `image` MSRV
exceeds the workspace setting and must be addressed before installation.

### Transitive Dependency Risk

Image codecs can bring many transitive dependencies, especially with default
formats and AVIF/WebP support. Installation issues should choose a narrow
feature set and include `cargo tree -e features`.

### Unsafe or FFI Exposure

Kinetik code must remain unsafe-free. Avoid native codec features unless
explicitly approved with ADR 0017 safety review.

### Platform Support

Initial support is desktop editor/import workflow. Runtime targets should
consume imported outputs and bundles rather than parsing arbitrary source assets
at frame time.

### Build-Time Impact

Codec and glTF dependencies will increase build time. Keep parser dependencies
behind importer/resource boundaries.

### Runtime Size and Performance Impact

Importers are editor/build-time systems. Avoid placing source parser
dependencies on standalone runtime hot paths.

### Public API Impact

Public APIs should expose Kinetik asset references, import settings, import
records, and diagnostics, not third-party parser structs.

### Serialized Format Impact

Import settings and cache metadata affect generated and manifest-adjacent
formats. Contract changes require focused serialized-format review and tests.

### Crate Ownership

`kinetik-resource` owns initial import metadata and diagnostics. Dedicated
format importer modules or crates may be proposed later if importer code grows.

## Approval Outcome

Approved by maintainer direction in issue #48 on 2026-05-20.

Create separate installation issues before adding importer crates. Those issues
must resolve `image` MSRV/version selection, choose narrow format features, and
keep parser/decoder types behind import/resource boundaries.
