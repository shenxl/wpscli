---
name: wps-multipart_upload_tasks
version: 1.0.0
description: "WPS OpenAPI service: multipart_upload_tasks"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli multipart_upload_tasks --help"
    auth_types: ["app", "user"]
---

# multipart_upload_tasks service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli multipart_upload_tasks <endpoint> [flags]
```

## API Resources

### multipart_upload_tasks

  - `abort-multipart-upload-task` — 中止分块上传任务 (`GET` `/v7/multipart_upload_tasks/{upload_id}/abort`; scopes: `kso.file.readwrite, kso.appfile.readwrite, kso.file.readwrite`; auth: `both`)
  - `commit-multipart-upload-task` — 提交分块上传任务 (`GET` `/v7/multipart_upload_tasks/{upload_id}/commit`; scopes: `kso.file.readwrite, kso.appfile.readwrite, kso.file.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli multipart_upload_tasks --help
wpscli schema multipart_upload_tasks
```
