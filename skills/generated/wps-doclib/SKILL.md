---
name: wps-doclib
version: 1.0.0
description: "WPS OpenAPI service: doclib"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli doclib --help"
    auth_types: ["app", "user"]
---

# doclib service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli doclib <endpoint> [flags]
```

## API Resources

### doclib

  - `create-doclib` — 创建团队文档库 (`GET` `/v7/doclib/create`; scopes: `kso.doclib.readwrite, kso.doclib.readwrite`; auth: `both`)
  - `delete-doclib` — 删除团队文档库 (`GET` `/v7/doclib/delete`; scopes: `kso.doclib.readwrite, kso.doclib.readwrite`; auth: `both`)
  - `get-doclib` — 获取文档库信息 (`GET` `/v7/doclib/meta`; scopes: `kso.doclib.readwrite, kso.doclib.readwrite, kso.doclib.read`; auth: `both`)
  - `search-doclib` — 搜索团队文档库 (`GET` `/v7/doclib/search`; scopes: `kso.doclib.readwrite, kso.doclib.read, kso.doclib.readwrite, kso.doclib.read`; auth: `both`)

## Discovering Commands

```bash
wpscli doclib --help
wpscli schema doclib
```
