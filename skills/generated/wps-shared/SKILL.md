---
name: wps-shared
version: 1.1.0
description: "wps CLI: shared auth model, global flags, scope preflight and security rules."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    auth_types: ["app", "user", "cookie"]
---

# wpscli Shared Reference

## Authentication

### app auth (OpenAPI, preferred for org/application data)

```bash
wpscli auth setup --ak <AK> --sk <SK>
wpscli drives list-drives --auth-type app
```

### user auth (OpenAPI, preferred for user-owned files)

```bash
wpscli auth setup --ak <AK> --sk <SK>
wpscli auth login --user --print-url-only
wpscli auth login --user --code <authorization_code>
wpscli files search-files --auth-type user --query keyword=周报
```

### oauth daemon mode (recommended for long-running agents)

```bash
# start extracted daemon under tools/oauth-server
cd tools/oauth-server && ./start.sh --daemon

# point cli to oauth server
wpscli auth setup --oauth-server http://127.0.0.1:8089
wpscli auth status
```

### cookie auth (private V7, supplemental only)

Cookie mode is for private interfaces under `https://api.wps.cn` and is not production-stable.

```bash
# 1) full cookie
export WPS_CLI_COOKIE='wps_sid=...; csrf=...'

# 2) sid only
export WPS_CLI_WPS_SID='...'

# 3) cookie file (json or plain text)
export WPS_CLI_COOKIE_FILE=~/.cursor/skills/wpsv7-skills/.wps_sid_cache.json

# execute private endpoint
wpscli raw GET /v7/recent_chats --auth-type cookie
```

## Auth Selection Matrix

| Scenario | Recommended Auth | Why |
|----------|------------------|-----|
| Org directory / dept / member sync | `app` | Usually app-role scopes only |
| User-owned files and personal docs | `user` | Requires delegated user consent |
| Private V7 interfaces (IM search, recent chats) | `cookie` | Non-OpenAPI endpoints |
| Unknown endpoint with declared scopes | Follow endpoint `auth` tag in generated skill | Uses scope catalog matrix |

## Scope Preflight Recovery

When execution fails with scope/auth mismatch:

1. Run `wpscli auth status` to inspect token and auto-refresh readiness.
2. Switch auth type according to endpoint `auth` tag (`app`, `user`, `both`, `cookie-only`).
3. Re-login user token if delegated scope changed: `wpscli auth login --user`.
4. For cookie-only endpoints, refresh cookie source and retry with `--auth-type cookie`.

## Security Rules

1. Prefer `wpscli auth setup/login` encrypted local storage over long-lived env tokens.
2. Use `wpscli auth harden --apply` regularly.
3. Treat cookie credentials as high-risk ephemeral secrets; do not commit cookie files.
4. For destructive operations, use `--dry-run` first.
5. For bulk dbsheet writes, always use batch mode to avoid timeout and consistency issues.

## Global Flags

| Flag | Description |
|------|-------------|
| `--output <FORMAT>` | Output format: `json` (default), `compact`, `table` |
| `--dry-run` | Print request without sending API call |
| `--auth-type <app|user|cookie>` | Select auth mode |
| `--retry <N>` | Retry count for network failures |

## CLI Syntax

```bash
wpscli <service> <endpoint> [flags]
wpscli raw <METHOD> <PATH|URL> [flags]
wpscli schema <service> [endpoint]
wpscli ui all
wpscli guide
```
