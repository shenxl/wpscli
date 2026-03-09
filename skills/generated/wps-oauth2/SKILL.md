---
name: wps-oauth2
version: 1.0.0
description: "WPS OpenAPI service: oauth2"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli oauth2 --help"
---

# oauth2 service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli oauth2 <endpoint> [flags]
```

## API Resources

### oauth2

  - `flow` — 用户授权流程 (`GET` `/oauth2/auth`; scopes: `-`)
  - `get-user-access-token` — 获取用户access_token (`POST` `/oauth2/token`; scopes: `-`)
  - `isvapp-app-access-token` — 三方应用获取应用的access_token (`POST` `/oauth2/token`; scopes: `-`)
  - `isvapp-tenant-access-token` — 三方应用获取租户的access_token (`POST` `/oauth2/token`; scopes: `-`)
  - `push-app-ticket` — 推送app_ticket (`POST` `/oauth2/ticket/active`; scopes: `-`)
  - `refresh-user-access-token` — 刷新用户access_token (`POST` `/oauth2/token`; scopes: `-`)
  - `selfapp-tenant-access-token` — 自建应用获取租户的access_token (`POST` `/oauth2/token`; scopes: `-`)

## Discovering Commands

```bash
wpscli oauth2 --help
wpscli schema oauth2
```
