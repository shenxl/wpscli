---
name: wps-remove_workbench_favorite_app
version: 1.0.0
description: "WPS OpenAPI service: remove_workbench_favorite_app"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli remove_workbench_favorite_app --help"
    auth_types: ["user"]
---

# remove_workbench_favorite_app service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli remove_workbench_favorite_app <endpoint> [flags]
```

## API Resources

### remove_workbench_favorite_app

  - `remove-workbench-favorite-app` — 取消用户【常用】应用 (`GET` `/v7/remove_workbench_favorite_app`; scopes: `kso.app.readwrite`; auth: `user`)

## Discovering Commands

```bash
wpscli remove_workbench_favorite_app --help
wpscli schema remove_workbench_favorite_app
```
