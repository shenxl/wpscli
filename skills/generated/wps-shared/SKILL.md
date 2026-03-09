---
name: wps-shared
version: 1.0.0
description: "wps CLI: Shared patterns for authentication, global flags, and output formatting."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
---

# wpscli — Shared Reference

## Authentication

```bash
# Configure AK/SK and OAuth fields
wpscli auth setup --ak <AK> --sk <SK>

# Step 1: print OAuth consent URL and open it in browser
wpscli auth login --user --print-url-only

# Step 2: exchange authorization code for user token
wpscli auth login --user --code <authorization_code>
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--output <FORMAT>` | Output format: `json` (default), `compact`, `table` |
| `--dry-run` | Print request without sending API call |
| `--auth-type <app|user>` | Select app token or user token |
| `--retry <N>` | Retry count for network failures |

## CLI Syntax

```bash
wpscli <service> <endpoint> [flags]
wpscli raw <METHOD> <PATH|URL> [flags]
wpscli schema <service> [endpoint]
wpscli ui all
wpscli guide
```
