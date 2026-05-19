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
