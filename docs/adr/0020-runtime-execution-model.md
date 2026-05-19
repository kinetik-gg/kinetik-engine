# ADR 0020: Runtime Execution Model

## Status

Accepted as initial project direction.

## Context

Kinetik needs an executable runtime model before implementation begins. The
runtime must serve the editor, standalone game execution, scripting, diagnostics,
and future MCP workflows without leaking editor-only state or raw platform
capabilities into gameplay scripts.

## Decision

Kinetik is 3D-first, instance-scripted, and runtime/editor separated.

Initial runtime direction:

- 3D-first; 2D can follow later.
- Instance hierarchy is the gameplay authoring model.
- ECS-like internals may exist only as implementation details.
- Luau scripts use simple lifecycle hooks.
- The first runtime core should be deterministic and single-threaded.
- Standalone runtime must not depend on editor crates.
- Multiplayer networking, replication, raw sockets, and hosting are out of
  scope initially.

Initial script lifecycle:

```lua
function Ready() end
function Update(dt: number) end
function PhysicsUpdate(dt: number) end
function Exit() end
```

## Consequences

- Runtime scripting remains approachable.
- The initial runtime avoids multiplayer and replication complexity.
- The single-threaded core keeps early behavior deterministic while leaving room
  for later parallelism.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- Multiplayer networking as an early runtime feature.
  - Rejected because it would dominate architecture before the local runtime,
    scene, scripting, diagnostics, and editor loop are proven.
- Multithreaded runtime core from the beginning.
  - Rejected for the initial implementation because deterministic behavior and
    simple debugging matter more at this stage.

## Reopen Conditions

- Multiplayer or server runtime requirements become a near-term product goal.
- Performance needs require earlier parallelism than planned.
