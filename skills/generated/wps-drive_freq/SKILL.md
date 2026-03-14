---
name: wps-drive_freq
version: 1.0.0
description: "WPS OpenAPI service: drive_freq"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli drive_freq --help"
    auth_types: ["app", "user"]
---

# drive_freq service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli drive_freq <endpoint> [flags]
```

## API Resources

### drive_freq

  - `get-frequent-list` — 获取常用列表 (`GET` `/v7/drive_freq/items`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read`; auth: `both`)

## Discovering Commands

```bash
wpscli drive_freq --help
wpscli schema drive_freq
```
