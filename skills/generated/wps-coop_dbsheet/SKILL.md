---
name: wps-coop_dbsheet
version: 1.0.0
description: "WPS OpenAPI service: coop_dbsheet"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli coop_dbsheet --help"
---

# coop_dbsheet service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli coop_dbsheet <endpoint> [flags]
```

## API Resources

### dbsheet

  - `add-subjects` — 批量添加成员 (`GET` `/v7/coop/dbsheet/{file_id}/content_permissions/subjects/batch_add`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `bind-parent` — 绑定父子记录 (`GET` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/parents/{parent_id}/children/batch_bind`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `change-subjects` — 批量更改成员权限 (`GET` `/v7/coop/dbsheet/{file_id}/content_permissions/subjects/batch_change`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `create-field` — 创建字段 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/fields`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `create-permission` — 新增自定义角色 (`GET` `/v7/coop/dbsheet/{file_id}/content_permissions/batch_add/async_tasks/create`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `create-record` — 创建记录 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/create`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `create-sheet` — 创建工作表 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/create`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `create-view` — 创建视图 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/views`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `create-webhook` — 创建webhook (`POST` `/v7/coop/dbsheet/{file_id}/hooks/create`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `delete-field` — 删除字段 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/fields/delete`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `delete-permission` — 删除自定义角色 (`GET` `/v7/coop/dbsheet/{file_id}/content_permissions/delete/async_tasks/create`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `delete-record` — 删除记录 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/batch_delete`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `delete-sheet` — 删除工作表 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/delete`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `delete-view` — 删除视图 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/delete`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `delete-webhook` — 删除webhook (`POST` `/v7/coop/dbsheet/{file_id}/hooks/{hook_id}/delete`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `disable-parent` — 禁用父子关系（仅对前端生效） (`GET` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/parents/disable`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `enable-parent` — 启用父子关系（仅对前端生效） (`GET` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/parents/enable`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `get-schema` — 获取Schema (`GET` `/v7/coop/dbsheet/{file_id}/schema`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `list-child` — 查询子记录 (`GET` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/parents/{parent_id}/children`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `list-permission` — 列举自定义角色 (`GET` `/v7/coop/dbsheet/{file_id}/content_permissions`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `list-record` — 列举记录 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `list-record-by-page` — 按页列举记录 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/list_by_page`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `list-subjects` — 列举成员 (`GET` `/v7/coop/dbsheet/{file_id}/content_permissions/subjects`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `list-webhook` — 列举webhook (`GET` `/v7/coop/dbsheet/{file_id}/hooks`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `parent-status` — 查询父子关系是否禁用（仅对前端生效） (`GET` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/parents/status`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `remove-subjects` — 批量删除成员 (`GET` `/v7/coop/dbsheet/{file_id}/content_permissions/subjects/batch_remove`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `search-record` — 检索记录 (`GET` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/{record_id}`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.read, kso.dbsheet.readwrite, kso.dbsheet.read`)
  - `unbind-parent` — 解绑父子记录 (`GET` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/parents/{parent_id}/children/batch_unbind`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `update-field` — 更新字段 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/fields/update`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `update-record` — 更新记录 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/update`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `update-sheet` — 更新工作表 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/update`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)
  - `update-view` — 更新视图 (`POST` `/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/update`; scopes: `kso.dbsheet.readwrite, kso.dbsheet.readwrite`)

## Discovering Commands

```bash
wpscli coop_dbsheet --help
wpscli schema coop_dbsheet
```
