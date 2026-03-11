# WPSCLI Hybrid Auth Strategy (OpenAPI + Cookie)

## Decision

- Primary path: OpenAPI (`auth-type app|user`)
- Supplement path: Cookie session (`auth-type cookie`)
- Routing principle: `openapi_first`, then `cookie_fallback` only for capability gaps

## What is implemented now

1. `wpscli` supports `--auth-type cookie` in:
   - dynamic service commands (`wpscli <service> <endpoint> ...`)
   - `wpscli raw ...`
   - helper command option parsing (where `auth-type` exists)
2. Cookie credential loading priority:
   - `WPS_CLI_COOKIE` (full cookie string)
   - `WPS_CLI_WPS_SID` / `WPS_SID` / `wps_sid` (builds `wps_sid=<sid>; csrf=<sid>`)
   - `WPS_CLI_COOKIE_FILE`
   - fallback files:
     - `~/.config/wps/cookie.json`
     - `~/.config/wps/cookie_cache.json`
     - `~/.cursor/skills/wpsv7-skills/.wps_sid_cache.json`
     - `~/.cursor/skills/wps-sign/cookie.json`
3. Cookie transport defaults:
   - base: `https://api.wps.cn`
   - origin: `https://365.kdocs.cn`
   - referer: `https://365.kdocs.cn/woa/im/messages`
4. Cookie mode skips scope preflight claim parsing (JWT scope claim unavailable).

## Capability supplement matrix (vs `cursor-skills-wps365`)

| Domain | `wpscli` current strength | Cookie supplement value | Recommendation |
|---|---|---|---|
| doc / files / dbsheet | High (stable OpenAPI + helper workflows) | Low | Keep OpenAPI primary |
| IM (`chats/messages`) | Medium (helper is minimal) | High | Add cookie-enabled IM helper expansion |
| user current / contacts search | Medium (org helper is app-oriented) | High | Add `user-current` + contact search shortcuts on cookie route |
| calendar / meeting | Medium-High (APIs available but helper not full) | Medium | Prefer OpenAPI, allow cookie fallback for blocked operations |
| sign/status (`/woa/api/...`) | Low/None | Very High | Cookie-only helper (`sign`) is appropriate |

## Proposed phase plan

### Phase 1 (done in this round)
- Add cookie auth as third mode in core executor and command layer.

### Phase 2 (next)
- Add capability registry (`provider=openapi|cookie`, `fallback=true|false`).
- Expand helpers:
  - `chat`: recent/search/history/send/recall
  - `user`: current/search shortcuts
  - optional `sign` helper for work status

### Phase 3
- Unified error model for agent recovery:
  - `auth_expired`, `csrf_failed`, `scope_mismatch`, `param_invalid`
- Structured routing telemetry:
  - `source=openapi|cookie`, success rate, retry count, fallback rate

## Guardrails

- For critical automation chains, keep OpenAPI-only policy by default.
- Cookie mode should be explicitly marked in operation logs for auditability.
- Do not silently replace OpenAPI with cookie unless fallback is declared.
