# Signal Bus Contract

## Purpose

Define deterministic signal and event delivery before M9 implementation adds
public signal APIs, Luau-facing event contracts, physics event routing, or
runtime flush integration.

Signals are runtime-facing coordination primitives. They must be predictable
enough for scripts, physics, diagnostics, MCP inspection, and tests without
letting any subsystem invent a private event path.

## Owning Crates

- `kinetik-signal`: signal descriptors, connection handles, delivery queues,
  deterministic ordering, and signal-domain diagnostics.
- `kinetik-app` or the future runtime owner crate: frame phase orchestration,
  fixed-step/frame-level flush calls, runtime world teardown hooks, and
  diagnostics/log attribution.
- `kinetik-script`: future Luau binding adapter over Kinetik-owned signal
  handles and event payload contracts.
- `kinetik-physics`: future producer of fixed-step collision/contact events
  through Kinetik-owned event records, not backend-native event types.
- `kinetik-editor` and MCP surfaces: inspect signal state through runtime/editor
  APIs; they do not own signal delivery.

## Signal Descriptor Contract

A signal descriptor represents an author-facing event source.

Descriptors must provide:

- Stable runtime `SignalId`.
- Stable author-facing `PascalCase` name where exposed to scripts or reflection.
- Owning runtime instance when the signal belongs to an instance.
- Flush domain: frame-level or fixed-step.
- Optional payload schema once property/value payloads are implemented.

Descriptor registration must be deterministic. If registration order affects
IDs, tests must assert the order. Duplicate author-facing names are invalid
within the same owner and should produce structured diagnostics once diagnostics
are wired into the signal crate.

## Connection Contract

Connections represent subscriptions to one signal descriptor.

Connections must provide:

- A typed connection handle owned by `kinetik-signal`.
- The target `SignalId`.
- Deterministic creation order used as the default delivery tie-breaker.
- Lifecycle state: connected, disconnected, or invalidated by owner teardown.

Disconnecting is idempotent. Delivery skips disconnected connections. Teardown
of an owning instance or runtime world invalidates related connections without
running callbacks during teardown.

Future Luau bindings may expose RBXScriptSignal-like `Connect` and `Disconnect`
methods, but raw VM callback handles must stay behind `kinetik-script-luau`.

## Event Queue Contract

Events are queued records delivered at explicit flush points.

Queued events must record:

- Target `SignalId`.
- Flush domain: frame-level or fixed-step.
- Emit sequence number within that domain.
- Optional runtime frame index.
- Optional fixed-step index for fixed-step events.
- Optional emitter runtime instance ID and edit GUID mapping when available.
- Payload values once payload schemas exist.

Delivery order is deterministic:

1. Flush domain selected by the runtime frame phase.
2. Frame index, then fixed-step index when applicable.
3. Emit sequence within that domain.
4. Signal registration order.
5. Connection creation order.

Callbacks must not mutate connection collections while they are being iterated.
Connect, disconnect, and structural changes requested during delivery are
queued and applied at safe sync points.

## Frame Integration

Signal delivery follows ADR 0022 and `runtime-frame.md`:

- Fixed-step events flush during the fixed-step signal/event phase after physics
  event collection.
- Frame-level events flush during the frame-level signal/event phase after all
  fixed steps complete.
- Structural changes requested by signal handlers are applied only at the
  approved safe structural-change sync points.
- Rendering observes state only after signal flushes and derived-state updates
  complete.

Signal APIs must not define an alternate frame loop or apply structural changes
immediately during iteration.

## Diagnostics Behavior

Signal diagnostics should use structured diagnostics with stable codes.

Initial invalid states to report:

- Duplicate signal names for the same owner.
- Invalid signal handles.
- Invalid or stale connection handles.
- Payload/schema mismatch once payload schemas exist.
- Event emitted to a signal whose owner has been destroyed.

Diagnostics should include signal name, `SignalId`, connection handle when
applicable, runtime frame index, fixed-step index when applicable, and owning
instance GUID/path when available.

## Dependency Boundaries

No new dependency is required for the initial signal contract.

Signal public APIs expose Kinetik-owned handles, descriptors, event records, and
diagnostics. They must not expose Luau VM, physics backend, editor UI, MCP, or
platform event-loop types.

## Serialized-Format Impact

No serialized-format impact is approved by this contract.

Runtime connection state and queued events are runtime-only. Future persisted
authoring data, such as custom signal declarations, requires a dedicated
serialization issue and migration/golden-test plan.

## Public API Constraints

Implementation issues may add focused `kinetik-signal` APIs only when their
names and behavior follow this contract.

Public APIs should:

- Use typed Kinetik handles rather than raw integers.
- Keep author-facing script names PascalCase.
- Preserve deterministic ordering in return values and delivery.
- Keep runtime/editor/MCP boundaries separate.
- Provide meaningful errors or diagnostics instead of silently ignoring invalid
  handles.

If implementation needs payload schemas, custom signal authoring, cross-thread
delivery, async delivery, or serialized signal declarations, stop for a focused
ADR/spec update before coding.

## Follow-Up Issues

- M9 signal descriptor and connection handle model.
- M9 deterministic frame-level event queue.
- M9 fixed-step event queue and frame flush integration.
- M9 disconnect, owner teardown, and cleanup behavior.
- M9 signal diagnostics.
