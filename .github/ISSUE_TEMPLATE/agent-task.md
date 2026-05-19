---
name: Agent task
about: Scoped, agent-executable Kinetik task
title: ""
labels: "status: needs-triage"
assignees: ""
---

## Mission

One sentence describing the outcome.

## Scope

Exact files, crates, systems, commands, or behavior the agent may change.

## Do Not Touch

Explicit exclusions, especially adjacent systems, public APIs, serialized
formats, dependencies, unsafe code, and unrelated refactors.

## Inputs

Links to roadmap item, design note, reproduction, fixture, ADRs, screenshots, or
prior issue.

## Expected Outputs

Concrete deliverables: code, tests, docs, diagnostics, fixtures, command
behavior, UI behavior, etc.

## Implementation Level

Level 0 / 1 / 2 / 3 / 4 / 5.

## Required Tests / Checks

Exact commands and expected verification layer.

## Relevant Docs / ADRs

List only the ADRs expected to govern the task.

## Architecture / Approval Gates

- Public API change: yes/no
- Serialized format change: yes/no
- Dependency change: yes/no
- Unsafe Rust: yes/no
- Editor/runtime boundary affected: yes/no
- Human approval required before implementation: yes/no

## Human Verification

What a maintainer must judge manually, if anything.

## Open Questions

Questions that block execution. If none, write `None`.

## Definition of Done

- [ ] Code compiles.
- [ ] Required checks pass.
- [ ] Tests cover behavior.
- [ ] No unrelated files changed.
- [ ] Errors/diagnostics are meaningful.
- [ ] Docs updated if behavior changed.
- [ ] Relevant ADRs mentioned in PR notes.
