# Template Project Contract

First-party Kinetik templates are product acceptance targets. They live under
`templates/`, not `examples/`.

`examples/` remains available for small illustrative code or document samples
that are useful to read but are not acceptance targets. A template must be a
complete project-shaped workspace that can eventually be opened by Kinetik
Studio, driven by MCP/headless checks, rendered, saved, and reviewed.

## Repository Layout

Each first-party template uses this shape:

```text
templates/
  template-name/
    README.md
    VERIFY.md
    Kinetik.toml
    scenes/
      main.knscene
    project/
      assets.knmanifest
    assets/
    scripts/
```

`README.md` explains the template purpose and current feature contract.
`VERIFY.md` records deterministic verification expectations. Source files use
the normal project layout from `docs/architecture/project-organization.md`.

Do not commit `.kinetik/`, build output, editor local state, generated caches,
or machine-specific screenshot captures inside a template.

## README Format

Every template README must include:

- `# <Template Name>`
- `Purpose`: one short paragraph describing what the template proves.
- `Acceptance Target`: the milestone or issue that owns the template.
- `Engine Features`: a short list of engine systems intentionally exercised.
- `Not Covered`: explicit exclusions so templates do not silently expand scope.
- `Human Verification`: the visual or interaction checks a reviewer should run.

README files should avoid tutorial prose until the template is stable. Templates
first prove engine behavior; later docs can teach from them.

## Verification Notes

Every template `VERIFY.md` must include:

- `Required Checks`: exact commands or named CI jobs.
- `Headless/MCP Checks`: semantic checks that do not require a display.
- `Golden Files`: committed deterministic files that must not drift.
- `Screenshots`: expected capture names, viewpoint, size, and tolerance.
- `Manual Review`: concise human checks that automation cannot yet cover.
- `Known Gaps`: bounded follow-up issue links for missing automation.

Verification notes must name the owning issue for any skipped screenshot,
golden, MCP, or human check. A template cannot claim a behavior as covered
without either automation or an explicit manual verification step.

## Golden Policy

Golden files are required when a template depends on stable serialized source,
generated manifests, diagnostics, or deterministic render/screenshot outputs.

Golden files must be:

- Text-first when practical.
- Deterministically ordered.
- Small enough for code review.
- Updated in the same PR as the behavior that changes them.
- Paired with a test or verification note explaining how to regenerate or
  inspect them.

Binary golden files are allowed only for screenshots or compact reference
assets needed by a template. Prefer deterministic text manifests and source
documents over opaque binaries.

## Screenshot Policy

Screenshots are acceptance evidence for visual templates, not decoration.

For each screenshot, `VERIFY.md` must record:

- Capture command or manual procedure.
- Viewport/camera name.
- Resolution.
- Platform assumptions.
- Tolerance policy, if automated comparison exists.

Until automated screenshot comparison exists, screenshots are human-verification
artifacts. They must not be used as the only proof for nonvisual behavior that
can be checked with headless tests.

## Determinism Rules

Template fixtures must avoid nondeterministic data:

- No wall-clock timestamps in committed source or goldens.
- No random IDs unless seeded and documented.
- No absolute local filesystem paths.
- No user-specific editor state.
- No external network dependency for default verification.
- No generated cache files.

Runtime or script behavior used by templates must either be deterministic or
called out in `VERIFY.md` with a bounded follow-up issue.

## CI and Human Verification

CI should run the narrowest reliable checks first: format/lints, unit tests,
template fixture roundtrips, headless/MCP verification, then screenshot checks
when available.

Human verification remains required when a milestone explicitly asks for it or
when the relevant visual automation does not exist yet. The PR that introduces
or materially changes a template must include the human verification result in
the PR body.
