---
name: wps-deleted_files
version: 1.0.0
description: "WPS OpenAPI service: deleted_files"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli deleted_files --help"
---

# deleted_files service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli deleted_files <endpoint> [flags]
```

## API Resources

### deleted_files

  - `batch-delete-files` — 一级回收站批量文件删除 (`GET` `/v7/deleted_files/batch_delete`; scopes: `kso.deleted_file.readwrite, kso.deleted_file.readwrite`)
  - `batch-restore-files` — 批量还原回收站文件 (`GET` `/v7/deleted_files/batch_restore`; scopes: `kso.deleted_file.readwrite, kso.deleted_file.readwrite`)
  - `clear-files` — 清空回收站文件 (`GET` `/v7/deleted_files/clear`; scopes: `kso.deleted_file.readwrite, kso.deleted_file.readwrite`)
  - `delete-file` — 删除一级回收站文件 (`GET` `/v7/deleted_files/{file_id}/delete`; scopes: `kso.deleted_file.readwrite, kso.deleted_file.readwrite`)
  - `get-file-meta` — 获取回收站文件信息 (`GET` `/v7/deleted_files/{file_id}/meta`; scopes: `kso.deleted_file.read, kso.deleted_file.read`)
  - `list-files` — 获取回收站文件列表 (`GET` `/v7/deleted_files`; scopes: `kso.deleted_file.read, kso.deleted_file.read`)
  - `restore-file` — 还原回收站文件 (`GET` `/v7/deleted_files/{file_id}/restore`; scopes: `kso.deleted_file.readwrite, kso.deleted_file.readwrite`)

## Discovering Commands

```bash
wpscli deleted_files --help
wpscli schema deleted_files
```
