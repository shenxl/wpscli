---
name: wps-create_workbench_favorite_app
version: 1.0.0
description: "WPS OpenAPI service: create_workbench_favorite_app"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli create_workbench_favorite_app --help"
---

# create_workbench_favorite_app service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli create_workbench_favorite_app <endpoint> [flags]
```

## API Resources

### create_workbench_favorite_app

  - `create-workbench-favorite-app` — 添加用户【常用】应用 (`GET` `/v7/create_workbench_favorite_app`; scopes: `kso.app.readwrite`)

## Discovering Commands

```bash
wpscli create_workbench_favorite_app --help
wpscli schema create_workbench_favorite_app
```
