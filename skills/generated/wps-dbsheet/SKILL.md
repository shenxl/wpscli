---
name: wps-dbsheet
version: 1.0.0
description: "WPS OpenAPI service: dbsheet"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli dbsheet --help"
    auth_types: ["app", "user"]
---

# dbsheet service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli dbsheet <endpoint> [flags]
```

## API Resources

### dbsheet

  - `batch-create-sheet` — 批量创建工作表 (`GET` `/v7/dbsheet/{file_id}/sheets/batch_create`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `batch-delete-sheet` — 批量删除工作表 (`GET` `/v7/dbsheet/{file_id}/sheets/batch_delete`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `copy-dashboard` — 复制仪表盘 (`GET` `/v7/dbsheet/{file_id}/dashboards/{dashboard_id}/copy`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `disable-share` — 关闭分享视图 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/sharedlinks/{share_id}/close`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `enable-share` — 打开分享视图 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/sharedlinks/open`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `get-meta` — 获取表单元数据 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/forms/{view_id}/meta`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`; auth: `both`)
  - `get-repeatable` — 查询表单视图是否可以重复提交 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/sharedlinks/{share_id}/settings`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`; auth: `both`)
  - `get-status` — 查询视图是否开启分享 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/sharedlinks/status`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`; auth: `both`)
  - `get-view` — 获取视图 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`; auth: `both`)
  - `list-dashboard` — 列出仪表盘 (`GET` `/v7/dbsheet/{file_id}/dashboards`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`; auth: `both`)
  - `list-fields` — 列出表单问题 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/forms/{view_id}/fields`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`; auth: `both`)
  - `list-view` — 列出视图 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`; auth: `both`)
  - `set-repeatable` — 设置表单视图是否可以重复提交 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/sharedlinks/{share_id}/settings`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `update-fields` — 更新表单问题 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/forms/{view_id}/fields/{field_id}/update`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `update-meta` — 更新表单元数据 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/forms/{view_id}/meta`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)
  - `update-permission` — 修改分享权限 (`GET` `/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/sharedlinks/{share_id}/update`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli dbsheet --help
wpscli schema dbsheet
```
