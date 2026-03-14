---
name: wps-files
version: 1.0.0
description: "WPS OpenAPI service: files"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli files --help"
    auth_types: ["app", "user"]
---

# files service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli files <endpoint> [flags]
```

## API Resources

### files

  - `batch-get-file-info` — 批量获取文件信息 (`GET` `/v7/files/batch_get`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read`; auth: `both`)
  - `get-file-info` — 获取文件信息 (`GET` `/v7/files/{file_id}/meta`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read, kso.mcp.readwrite`; auth: `both`)
  - `get-permission-settings` — 获取文件权限配置项 (`GET` `/v7/files/{file_id}/permission_settings`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `post-permission-settings` — 设置文件权限配置项 (`GET` `/v7/files/{file_id}/permission_settings`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `query-ai-search-drive-status` — 查询驱动盘AI搜索状态 (`GET` `/v7/files/ai_search/query_drive_status`; scopes: `kso.file.search, kso.file_search.readwrite, kso.file.search`; auth: `user`)
  - `search-file` — 文件搜索 (`GET` `/v7/files/search`; scopes: `kso.file.search, kso.file_search.readwrite, kso.file.search, kso.mcp.readwrite`; auth: `user`)

## Discovering Commands

```bash
wpscli files --help
wpscli schema files
```
