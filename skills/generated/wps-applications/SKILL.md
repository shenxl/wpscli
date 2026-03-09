---
name: wps-applications
version: 1.0.0
description: "WPS OpenAPI service: applications"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli applications --help"
---

# applications service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli applications <endpoint> [flags]
```

## API Resources

### applications

  - `applications` — 获取用户【可用】应用列表 (`GET` `/v7/applications`; scopes: `kso.app.read`)

## Discovering Commands

```bash
wpscli applications --help
wpscli schema applications
```
