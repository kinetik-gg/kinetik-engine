# Testing Strategy

Required baseline checks:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

Test types:

- Unit tests.
- Serialization roundtrips.
- Golden fixtures.
- Invalid handle tests.
- Diagnostic code tests.
- Determinism tests.
- Integration vertical slices.
- Shader graph codegen tests.
- Physics event tests.
- Bundle verification tests.
- Unsafe-boundary tests for any approved unsafe exception.

## Automation Order

Use the narrowest reliable layer first:

1. Rust unit tests for deterministic logic.
2. Integration tests and fixtures for cross-crate behavior.
3. Golden fixtures for serialization, manifests, generated output, and stable
   diagnostics.
4. Headless runtime/editor tests when available.
5. MCP semantic tests once the relevant MCP server/tools exist.
6. Screenshot or visual checks for UI, viewport, rendering, and interaction.

MCP is the preferred semantic automation target once implemented, but it is not
a blocker for earlier milestones. Until MCP tools exist for a behavior, use
unit, integration, fixture, golden, or headless checks.

## Serialization Fixtures

Kinetik source formats are Git-friendly, deterministic, and text-first. Any
issue that changes serialized source shape, generated format, or migration
behavior must include or update golden fixtures unless an ADR explicitly says
otherwise.

Minimum expectations for early serialization work:

- Roundtrip tests for valid project, scene, prefab, and manifest examples.
- Golden files for deterministic field ordering and formatting.
- Invalid fixture tests for duplicate IDs, unknown classes/properties, malformed
  paths, and missing required fields.
- Migration notes whenever an existing fixture format changes.

Do not defer serialization validation until editor integration; late format
changes are expensive and risk corrupting project source.
