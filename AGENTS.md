# Kinetik Agent Charter

You are contributing to Kinetik Engine.

## Prime Directive

Build Kinetik as a cohesive, polished, maintainable game engine. Every contribution must improve the engine without damaging architectural clarity, editor quality, runtime stability, or future extensibility.

Agents are not here to generate code volume. Agents are here to ship correct, elegant, integrated systems.

## Mandatory Rules

- No silent assumptions.
- If in doubt, ask with options and a recommendation.
- Do not touch unrelated code.
- Do not add dependencies without approval, rationale, boundary ownership, and license/safety review.
- Do not alter public APIs without approval.
- Do not alter serialized formats without an ADR/migration plan.
- Do not introduce `unsafe`, relax unsafe lints, or hide unsafe behind wrappers without an ADR and explicit human approval.
- Do not perform broad refactors during feature work.
- Keep patches small, focused, and reviewable.
- Write tests for behavior.
- Keep editor and runtime boundaries clean.
- Prefer clear names and simple code over clever abstractions.
- Avoid files above roughly 500-700 lines unless explicitly justified.
- Avoid 3+ levels of nesting; use guard clauses and extracted functions.
- No god objects. No god files. No vague `utils.rs` dumping grounds.

## Development Model

Kinetik work is issue-based and backlog-driven.

- Roadmap lives in docs.
- Backlog lives in GitHub issues or local backlog files until GitHub is ready.
- One issue maps to one branch/worktree and one focused patch.
- Agents claim scoped issues; they do not improvise broad work.
- Multiple agents may work in parallel only on disjoint issues and isolated worktrees.
- No multiple agents edit the same working directory.
- Architecture, public API, dependency, unsafe, serialized-format, and merge decisions remain human-supervised.
- Reviewer/QA agents should attack patches before integration.
- Integration happens only after checks pass and scope is confirmed.

Good issues are small enough for one focused agent and must use the task
contract below.

## ADR Usage

Agents must check relevant ADRs before changing architecture-sensitive areas.

Do not bulk-read every ADR by default. Instead:

1. Identify the task area.
2. Search ADR titles and docs for relevant keywords.
3. Read only the ADRs that could govern the task.
4. Mention the relevant ADRs in the task response or PR notes.
5. If the task conflicts with an accepted ADR, stop and ask for human direction.

Architecture-sensitive areas include:

- Project layout and workspace scaffolding.
- Scene, instance, prefab, package, and serialization behavior.
- Reflection, properties, scripting API, and public APIs.
- Editor commands, undo/redo, diagnostics, MCP, and play mode.
- Assets, manifests, import cache, bundles, and resource identity.
- Dependencies, unsafe Rust, runtime/editor boundaries, and generated formats.

If no relevant ADR exists and the task would set lasting direction, ask whether
to create one before implementing.

## Commit Style

Use Conventional Commits for commit messages:

```text
type(scope): summary
```

Examples:

```text
feat(scene): add root Game instance scaffold
fix(resource): preserve asset guid on manifest path update
docs(adr): lock diagnostics model
test(scene): add serialization roundtrip fixture
chore(ci): require clippy warnings as errors
```

Preferred types:

- `feat`: user-facing or engine capability.
- `fix`: bug fix.
- `docs`: documentation-only change.
- `test`: test-only change.
- `refactor`: behavior-preserving code restructure.
- `chore`: maintenance, tooling, or repository housekeeping.
- `ci`: continuous integration changes.
- `perf`: performance improvement.

Commit summaries must be imperative, concise, and specific. Do not mix unrelated
changes in one commit.

## Task Contract

Every task must include:

```text
Mission:
Scope:
Do not touch:
Inputs:
Expected outputs:
Required tests/checks:
Relevant docs:
Open questions:
Definition of done:
```

## Definition of Done

A task is done only when:

- Code compiles.
- Tests pass.
- No unrelated files changed.
- Public APIs are documented.
- Errors are meaningful.
- Assumptions are recorded.
- Docs are updated when behavior changes.
- Integration points are clear.
