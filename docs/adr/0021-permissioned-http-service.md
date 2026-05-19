# ADR 0021: Permissioned HTTP Service

## Status

Accepted as initial project direction.

## Context

Kinetik should support basic outbound HTTP requests from scripts, but HTTP is a
security and trust boundary. Requests must be sandboxed, permissioned, and
traceable to the script and instance that caused them.

HTTP must not become a general OS, socket, credential, filesystem, or platform
escape hatch.

## Decision

Kinetik provides a permissioned `HttpService` for outbound client requests.

Scripts request HTTP through Kinetik APIs. The engine enforces project policy,
limits, diagnostics, platform behavior, and provenance. Scripts do not receive
raw socket, OS, credential, cookie, keychain, environment, or filesystem access.

HTTP rules:

- HTTP must be explicitly enabled in project settings.
- Only outbound client requests are allowed.
- No raw sockets.
- No listening servers.
- No host credentials or environment access.
- Requests are asynchronous.
- Request timeout is required.
- Response body size limit is required.
- Redirect policy is explicit.
- Initial methods are limited to `GET` and `POST`.
- Headers are filtered or sanitized.
- Editor, play, and test modes may mock or deny HTTP.
- Blocked, timed out, oversized, or failed requests emit diagnostics.

## HTTP Provenance

Every HTTP request must carry script provenance.

Required provenance:

- Script asset GUID.
- Script `res://` path.
- Source file/range when available.
- Owning instance GUID.
- Owning instance scene path.
- Execution context: edit, play, or test.
- Call timestamp.
- Request method.
- Target URL origin.
- Permission decision.

Requests without provenance are denied and produce diagnostics.

Example trace:

```text
HTTP GET https://api.example.com/items
Called by:
  Script: res://scripts/EnemySpawner.luau
  Instance: /Game/Workspace/EnemySpawner
  Line: 42
Decision:
  Allowed by project HTTP policy
```

Example blocked trace:

```text
Blocked HTTP POST https://unknown.example
Called by:
  Script: res://scripts/Analytics.luau
  Instance: /Game/Scripts/Analytics
Reason:
  Origin not allowed by project HTTP policy
```

Suggested diagnostic codes:

```text
KT_HTTP_BLOCKED_ORIGIN
KT_HTTP_MISSING_PROVENANCE
KT_HTTP_TIMEOUT
KT_HTTP_RESPONSE_TOO_LARGE
KT_HTTP_METHOD_NOT_ALLOWED
```

## MCP and Editor Relationship

The editor and MCP should be able to inspect HTTP policy and request traces:

```text
http.list_requests
http.get_request_trace
http.list_policy
http.set_policy
```

Policy changes must go through editor commands and be represented in project
settings or manifests.

## Consequences

- Basic HTTP is available without exposing raw platform power.
- HTTP behavior is auditable by humans and agents.
- Suspicious or blocked network behavior can point to the responsible script and
  instance.
- Export targets can map `HttpService` to platform-specific HTTP backends.
- Reopening requires a follow-up ADR with alternatives and migration
  implications.

## Alternatives Considered

- No HTTP support initially.
  - Rejected because basic HTTP is useful enough for real projects and can be
    provided safely with strict provenance and policy.
- Raw network/socket access for scripts.
  - Rejected because it creates security, portability, and auditability risks.
- Anonymous global HTTP access.
  - Rejected because request attribution is required for trust, diagnostics, and
    security review.

## Reopen Conditions

- HTTP provenance cannot be captured reliably through the script runtime.
- Export platforms require a materially different HTTP policy model.
- Server runtime requirements need broader network capabilities.
