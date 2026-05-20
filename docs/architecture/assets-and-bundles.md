# Assets and Bundles

Source assets and centralized `.knmanifest` files are committed where reasonable.
Generated import cache and build output are ignored.

```text
Source asset -> Import cache -> Runtime bundle
```

Asset identity lives in `project/assets.knmanifest`. Assets are referenced by
stable GUID plus readable `res://` path. The GUID is durable identity; the path
is the project location and repair hint.

Import cache under `.kinetik/` is disposable and keyed by source hash, importer
identity/version, import settings, asset GUID, and cache schema version. Missing,
moved, duplicated, or invalid assets produce structured diagnostics instead of
silent identity replacement.

Runtime bundles use `.knbundle` and may be local, CDN/S3-hosted, or mod/package content. Remote bundles require hash verification and optional signatures.
