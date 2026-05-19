# Scripting Model

Kinetik uses Luau as the intended scripting language.

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
function ready() end
function update(dt: number) end
function physics_update(dt: number) end
function exit() end
```

`update(dt)` runs in the variable frame phase. `physics_update(dt)` runs during
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
