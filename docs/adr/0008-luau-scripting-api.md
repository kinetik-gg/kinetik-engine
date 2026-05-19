# ADR 0008: Luau Scripting API

## Status

Accepted as initial project direction.

## Context

Kinetik is a marathon-scale game engine intended to be built by coordinated
human and AI-agent contributors.

The scripting API should minimize friction for Unity and Roblox developers. It
should feel familiar to Luau users and avoid novelty naming unless it removes
real ambiguity.

## Decision

Typed creator-facing API.

Kinetik Luau API conventions:

- Prefer Roblox/Luau-familiar service access and colon methods.
- Use PascalCase for public lifecycle hooks, methods, properties, services, and
  reflected property paths.
- Use service globals for common entry points: `game`, `workspace`, `prefabs`,
  `assets`, `http`, `input`, `physics`, `audio`, `ui`, `time`, `tasks`,
  `debug`, `mathf`, and `script`.
- Use safe high-level services instead of raw engine/platform access.
- Use RBXScriptSignal-like events with `Connect` and `Disconnect`.
- Do not expose raw Rust memory, renderer internals, physics internals, OS
  handles, sockets, credentials, or filesystem access.

Initial lifecycle:

```lua
function Ready() end
function Update(dt: number) end
function PhysicsUpdate(dt: number) end
function Exit() end
```

Initial instance API shape:

```lua
instance.Name
instance.Parent
instance.ClassName
instance.Transform
instance:GetChildren()
instance:FindFirstChild(name)
instance:WaitForChild(name)
instance:IsA(class_name)
instance:Clone()
instance:Destroy()
instance:AddTag(tag)
instance:RemoveTag(tag)
instance:HasTag(tag)
```

Initial event shape:

```lua
local connection = instance.Touched:Connect(function(other)
end)

connection:Disconnect()
```

Initial asset and HTTP shape:

```lua
local prefab = assets:Load("res://prefabs/enemy.ktprefab")
local response = http:GetAsync("https://api.example.com/items")
```

Reflection and serialization keep canonical PascalCase property paths. Luau may
add aliases only when they map clearly to canonical reflected properties.

## Consequences

- Unity and Roblox developers encounter familiar naming and object access.
- Generated Luau bindings should preserve canonical reflected property names.
- API novelty is discouraged unless it materially improves clarity.
- This decision shapes crate boundaries, public APIs, editor workflows, tests,
  and agent assignments.
- Reopening requires a follow-up ADR with alternatives and migration implications.

## Alternatives Considered

- Kinetik-specific snake_case API.
  - Rejected because it increases friction for Roblox and Unity developers.
- Godot-style underscored lifecycle hooks.
  - Rejected because Kinetik is using Luau and wants a Roblox/Unity-familiar
    creator API.
- Expose lower-level engine subsystem APIs directly.
  - Rejected because scripts should use safe high-level services.

## Reopen Conditions

- Technical blocker discovered.
- Better option emerges with clear migration path.
- Product direction changes.
