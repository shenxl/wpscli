---
name: wps-service_principals
version: 1.0.0
description: "WPS OpenAPI service: service_principals"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli service_principals --help"
    auth_types: ["app", "user"]
---

# service_principals service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli service_principals <endpoint> [flags]
```

## API Resources

### service_principals

  - `get-sp` — 获取应用当前服务主体信息 (`GET` `/v7/service_principals/current`; scopes: `-`; auth: `both`)

## Discovering Commands

```bash
wpscli service_principals --help
wpscli schema service_principals
```
