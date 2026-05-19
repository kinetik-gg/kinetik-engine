# ADR 0004: Project Organization

## Status

Accepted as initial project direction.

## Context

Kinetik is a marathon-scale game engine intended to be built by coordinated human and AI-agent contributors.

## Decision

Kinetik projects use a file-based workspace on disk.

Users only need to pick or create a folder. Kinetik Studio scaffolds the project
workspace, creates the expected files and folders, and manages project structure
through editor commands.

Source content lives in normal folders. Stable IDs, import settings, and other
metadata are stored in a small set of centralized project manifests by domain
instead of Unity-style sidecar metadata files for every item.

Generated cache, import output, and build artifacts live under `.kinetik/` and
are disposable.

## Consequences

- Project state remains Git-friendly, reviewable, and agent-readable.
- The editor owns project scaffolding and should make the workspace usable
  without asking creators to manually arrange files.
- Manifests must be deterministic and structured enough to avoid unnecessary
  merge noise.
- Asset and instance GUIDs must survive renames and moves through manifest
  updates.
- This decision shapes crate boundaries, public APIs, editor workflows, tests,
  and agent assignments.
- Reopening requires a follow-up ADR with alternatives and migration implications.

## Alternatives Considered

- Per-file sidecar metadata.
  - Rejected as the default because it creates noisy workspaces and review
    surfaces.
- One centralized project document.
  - Rejected because it creates avoidable merge conflicts and makes targeted
    agent edits harder.
- Opaque editor database.
  - Rejected because it undermines Git workflows and external inspection.

## Reopen Conditions

- Technical blocker discovered.
- Better option emerges with clear migration path.
- Product direction changes.
