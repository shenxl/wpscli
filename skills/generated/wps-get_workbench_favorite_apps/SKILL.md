---
name: wps-get_workbench_favorite_apps
version: 1.0.0
description: "WPS OpenAPI service: get_workbench_favorite_apps"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli get_workbench_favorite_apps --help"
    auth_types: ["app", "user"]
---

# get_workbench_favorite_apps service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli get_workbench_favorite_apps <endpoint> [flags]
```

## API Resources

### get_workbench_favorite_apps

  - `get-workbench-favorites-apps` — 获取用户【常用】应用列表 (`GET` `/v7/get_workbench_favorite_apps`; scopes: `kso.app.read`; auth: `both`)

## Discovering Commands

```bash
wpscli get_workbench_favorite_apps --help
wpscli schema get_workbench_favorite_apps
```
