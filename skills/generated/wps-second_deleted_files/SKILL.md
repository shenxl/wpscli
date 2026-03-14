---
name: wps-second_deleted_files
version: 1.0.0
description: "WPS OpenAPI service: second_deleted_files"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli second_deleted_files --help"
    auth_types: ["app", "user"]
---

# second_deleted_files service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli second_deleted_files <endpoint> [flags]
```

## API Resources

### second_deleted_files

  - `batch-delete-second-deleted-files` — 二级回收站批量文件删除 (`GET` `/v7/second_deleted_files/batch_delete`; scopes: `kso.deleted_file.readwrite, kso.deleted_file.readwrite`; auth: `both`)
  - `delete-second-deleted-file` — 二级回收站单文件删除 (`GET` `/v7/second_deleted_files/{file_id}/delete`; scopes: `kso.deleted_file.readwrite, kso.deleted_file.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli second_deleted_files --help
wpscli schema second_deleted_files
```
