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
---

# files service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli files <endpoint> [flags]
```

## API Resources

### files

  - `batch-get-file-info` — 批量获取文件信息 (`GET` `/v7/files/batch_get`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read`)
  - `get-file-info` — 获取文件信息 (`GET` `/v7/files/{file_id}/meta`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read, kso.mcp.readwrite`)
  - `get-permission-settings` — 获取文件权限配置项 (`GET` `/v7/files/{file_id}/permission_settings`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`)
  - `post-permission-settings` — 设置文件权限配置项 (`GET` `/v7/files/{file_id}/permission_settings`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`)
  - `query-ai-search-drive-status` — 查询驱动盘AI搜索状态 (`GET` `/v7/files/ai_search/query_drive_status`; scopes: `kso.file.search, kso.file_search.readwrite, kso.file.search`)
  - `search-file` — 文件搜索 (`GET` `/v7/files/search`; scopes: `kso.file.search, kso.file_search.readwrite, kso.file.search, kso.mcp.readwrite`)

## Discovering Commands

```bash
wpscli files --help
wpscli schema files
```
