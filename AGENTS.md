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
- After a PR is merged, agents must clean up their temporary worktree and delete
  the issue branch both locally and on the remote. If a PR is squash-merged,
  verify the PR is merged before deleting because Git may not report the branch
  as merged by ancestry.

Good issues are small enough for one focused agent and must use the task
contract below.

## Agent Autonomy and Escalation

Agents may proceed without asking when all of these are true:

- The issue is scoped and matches the roadmap or an approved planning issue.
- The work is Level 0, Level 1, or Level 2.
- The patch is focused, reviewable, and does not cross architecture boundaries.
- No dependency is added or upgraded.
- No public API is removed, renamed, or semantically changed.
- No serialized source, generated format, migration, or golden fixture contract
  is changed.
- No `unsafe` is introduced, expanded, wrapped, or lint-suppressed.
- Editor/runtime/project/command/MCP boundaries remain as documented.
- Required tests/checks pass.

Agents may also proceed on Level 3 issues when the relevant ADRs and internal
API specs already define the cross-crate contract and the issue does not create
new architecture direction.

Agents must stop and ask for human approval when any of these apply:

- New dependency, dependency feature, or dependency version.
- Public API shape, naming, or compatibility change.
- Serialized format, migration, golden fixture contract, or source file layout
  change.
- Unsafe Rust, FFI boundary, unsafe lint change, or wrapper over unsafe code.
- Architecture decision not already covered by an accepted ADR/internal API
  spec.
- Editor behavior that would bypass engine/project/command/runtime APIs.
- Runtime behavior that could mutate saved edit/project state directly.
- Broad refactor mixed with feature work.
- File growth that would knowingly create or worsen a god file.

When escalation is needed, agents should present the options, recommend one,
and wait for approval before implementation.

## ADR Usage

Agents must check relevant ADRs before changing architecture-sensitive areas.

Do not bulk-read every ADR by default. Instead:

1. Identify the task area.
2. Check `docs/adr/README.md` for matching areas, keywords, and
   "read before touching" guidance.
3. Search ADR titles/docs for additional relevant keywords if the index is not
   enough.
4. Read only the ADRs that could govern the task.
5. Mention the relevant ADRs in the task response or PR notes.
6. If the task conflicts with an accepted ADR, stop and ask for human direction.

Architecture-sensitive areas include:

- Project layout and workspace scaffolding.
- Scene, instance, prefab, package, and serialization behavior.
- Reflection, properties, scripting API, and public APIs.
- Editor commands, undo/redo, diagnostics, MCP, and play mode.
- Assets, manifests, import cache, bundles, and resource identity.
- Dependencies, unsafe Rust, runtime/editor boundaries, and generated formats.

If no relevant ADR exists and the task would set lasting direction, ask whether
to create one before implementing.

## Implementation Levels

Every issue should declare an implementation level.

- Level 0: Docs/planning only.
- Level 1: Boilerplate or scaffold.
- Level 2: Deterministic core logic.
- Level 3: Cross-crate integration slice.
- Level 4: Runtime/editor behavior.
- Level 5: Visual, interaction, feel, or UX.

Level 2 is the default for engine code. Escalate when a task crosses crate
boundaries, changes serialization, affects editor/runtime behavior, or requires
human visual judgment.

Each issue should state:

```text
Implementation level:
Required tests/checks:
Human verification:
```

## Automation Protocol

Automate as much verification as practical, but use the right layer for the
task.

Preferred verification order:

1. Rust unit tests for deterministic logic.
2. Integration tests and fixtures for cross-crate behavior.
3. Golden fixtures for serialization, manifests, generated output, and stable
   diagnostics.
4. Headless runtime/editor tests when available.
5. MCP-driven editor/runtime tests when the MCP server exists.
6. Screenshot or visual regression checks for UI, viewport, rendering, and
   editor interaction.
7. Human review for gameplay feel, visual taste, animation timing, and UX
   judgment.

MCP should become the primary semantic automation harness for editor and runtime
work. Prefer MCP commands over pixel-level automation when both are available.
MCP is not a blocker before the relevant MCP server/tools exist; use unit,
integration, fixture, golden, and headless checks first.

Useful MCP test capabilities should include:

```text
project.open
project.create_temp
scene.list_instances
scene.create_instance
scene.set_property
diagnostics.list
play.start
play.step
play.stop
editor.get_dirty_state
editor.get_selection
editor.capture_viewport
```

Computer-use or OS-level UI automation is a fallback for app launch, menu/button
smoke tests, screenshots, and final UI verification. Do not rely on pixel-level
automation when a semantic API, fixture, unit test, integration test, or MCP
command can verify the behavior more directly.

Level guidance:

- Level 0: relevant docs/ADR consistency checks.
- Level 1: `cargo fmt --check` and `cargo test --workspace`.
- Level 2: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Level 3: Level 2 checks plus integration tests, fixtures, golden files, or `cargo doc --workspace --no-deps` when public APIs changed.
- Level 4: Level 3 checks plus lifecycle/order/command/diagnostic tests and MCP or headless runtime/editor smoke when available.
- Level 5: Level 4 checks plus screenshot/visual verification and human review.

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
