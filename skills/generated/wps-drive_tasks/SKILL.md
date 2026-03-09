---
name: wps-drive_tasks
version: 1.0.0
description: "WPS OpenAPI service: drive_tasks"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli drive_tasks --help"
---

# drive_tasks service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli drive_tasks <endpoint> [flags]
```

## API Resources

### drive_tasks

  - `get-file-task` — 获取异步任务信息 (`GET` `/v7/drive_tasks/{task_id}/meta`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read`)

## Discovering Commands

```bash
wpscli drive_tasks --help
wpscli schema drive_tasks
```
