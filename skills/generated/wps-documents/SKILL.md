---
name: wps-documents
version: 1.0.0
description: "WPS OpenAPI service: documents"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli documents --help"
---

# documents service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli documents <endpoint> [flags]
```

## API Resources

### documents

  - `batch-get-attachment-info` — 批量查询附件信息 (`GET` `/v7/documents/{file_id}/attachments/batch_get`; scopes: `kso.documents.readwrite, kso.documents.readwrite`)
  - `multiupload-1-get-upload-address` — 附件多段式上传-申请上传地址 (`GET` `/v7/documents/{file_id}/attachments/upload/address`; scopes: `kso.documents.readwrite, kso.documents.readwrite`)
  - `multiupload-3-complete-upload` — 附件多段式上传-提交上传完成 (`GET` `/v7/documents/{file_id}/attachments/upload/complete`; scopes: `kso.documents.readwrite, kso.documents.readwrite`)
  - `upload-attachment` — 上传附件 (`GET` `/v7/documents/{file_id}/attachments/upload`; scopes: `kso.documents.readwrite, kso.documents.readwrite`)

## Discovering Commands

```bash
wpscli documents --help
wpscli schema documents
```
