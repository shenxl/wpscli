---
name: wps-coop_files
version: 1.0.0
description: "WPS OpenAPI service: coop_files"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli coop_files --help"
    auth_types: ["app", "user"]
---

# coop_files service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli coop_files <endpoint> [flags]
```

## API Resources

### files

  - `batch-get-attachment-thumbnail` — 批量查询附件缩略图 (`GET` `/v7/coop/files/{file_id}/attachments/thumbnails/batch_get`; scopes: `kso.coop_files.readwrite, kso.coop_files.readwrite`; auth: `both`)
  - `get-attachment-info` — 查询附件信息 (`GET` `/v7/coop/files/{file_id}/attachments/{attachment_id}`; scopes: `kso.coop_files.readwrite, kso.coop_files.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli coop_files --help
wpscli schema coop_files
```
