# Scripting Model

Kinetik uses Luau as the accepted initial scripting language. ADR 0008 defines
the creator-facing Luau API direction, and this document does not replace or
soften that decision.

Future Kinetik language support from the sibling `kinetik-lang` repository is
planned as additive work. Lua/Luau scripts and future `.kn` scripts should be
able to coexist once a focused ADR, internal API spec, and implementation issue
define the integration. Until then, do not introduce `.kn` behavior or change
accepted Luau-facing APIs inside unrelated work.

Language-neutral engine contracts should stay neutral when they are not
specifically Luau bridge APIs:

- `kinetik-script` owns runtime-agnostic script lifecycle, attachment,
  diagnostics, provenance, and handle contracts.
- `kinetik-script-luau` owns Luau VM integration and Luau-specific API binding.
- Generic runtime, resource, project, editor, diagnostics, HTTP, hot-reload, and
  lifecycle APIs should avoid names that imply Luau is the only possible backend.
- Luau-specific namespaces, lifecycle syntax, idioms, and binding generation stay
  explicitly Luau-owned.
- Future language assets should carry language/backend provenance alongside
  asset GUIDs, `res://` paths, source ranges, owning instances, and execution
  context so diagnostics, permissions, and reload behavior remain auditable.

When touching scripting, runtime, project files, assets, editor integration,
diagnostics, permissions, HTTP, hot reload, or script lifecycle areas, review
whether names and contracts are accidentally single-backend. File scoped
follow-up issues for concrete preparation work instead of broadening unrelated
PRs. Any change that would alter accepted scripting ADRs, public APIs, or
serialized formats needs focused approval before implementation.

Initial namespaces:

```lua
game
workspace
prefabs
assets
http
input
physics
audio
ui
time
tasks
debug
mathf
script
```

Core types:

```lua
Vec2
Vec3
Vec4
Quat
Color
Transform
Rect
Aabb
```

Lifecycle:

```lua
function Ready() end
function Update(dt: number) end
function PhysicsUpdate(dt: number) end
function Exit() end
```

`Update(dt)` runs in the variable frame phase. `PhysicsUpdate(dt)` runs during
fixed simulation steps. Structural changes requested from scripts may be queued
until a safe runtime sync point.

Luau scripts use safe handles and high-level APIs. They never access raw Rust memory or engine internals.

HTTP access goes through Kinetik's permissioned runtime HTTP service. Requests
must be attributed to the calling script and owning instance; scripts do not get
raw socket, OS, credential, environment, or filesystem access.

Script-visible instance properties should be generated from runtime-owned
reflection metadata. Reflection and serialization keep canonical PascalCase
property paths; Luau may add idiomatic aliases only when they map clearly back to
canonical reflected properties.

Public Luau APIs should favor Roblox/Luau familiarity: PascalCase properties and
methods, colon method calls, service globals, and RBXScriptSignal-like events
with `Connect` and `Disconnect`.
