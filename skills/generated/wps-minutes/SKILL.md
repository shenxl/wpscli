---
name: wps-minutes
version: 1.0.0
description: "WPS OpenAPI service: minutes"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli minutes --help"
    auth_types: ["app", "user"]
---

# minutes service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli minutes <endpoint> [flags]
```

## API Resources

### minutes

  - `delete-imported-minute` — 删除导入生成的纪要 (`POST` `/v7/minutes/{minute_id}/delete`; scopes: `kso.meeting_minutes.readwrite`; auth: `app`)
  - `get-imported-minute` — 获取导入生成的纪要 (`GET` `/v7/minutes/{minute_id}`; scopes: `kso.meeting_minutes.read, kso.meeting_minutes.readwrite`; auth: `both`)
  - `import-minutes` — 导入录制文件生成纪要 (`POST` `/v7/minutes/create`; scopes: `kso.meeting_minutes.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli minutes --help
wpscli schema minutes
```
