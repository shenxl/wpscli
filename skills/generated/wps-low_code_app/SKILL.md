---
name: wps-low_code_app
version: 1.0.0
description: "WPS OpenAPI service: low_code_app"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli low_code_app --help"
---

# low_code_app service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli low_code_app <endpoint> [flags]
```

## API Resources

### low_code_app

  - `create-records` — 创建记录 (`POST` `/v7/low_code_app/app_instance/{app_instance_id}/table/{table_id}/records/create`; scopes: `kso.low_code_app.readwrite`)
  - `delete-records` — 删除记录 (`POST` `/v7/low_code_app/app_instance/{app_instance_id}/table/{table_id}/records/{record_id}/delete`; scopes: `kso.low_code_app.readwrite`)
  - `get-fields` — 获取数据库表字段信息 (`GET` `/v7/low_code_app/app_instance/{app_instance_id}/table/{table_id}/fields`; scopes: `kso.low_code_app.read, kso.low_code_app.readwrite`)
  - `get-members` — 获取应用下的所有成员 (`POST` `/v7/low_code_app/app_instance/{app_instance_id}/members`; scopes: `kso.low_code_app.read, kso.low_code_app.readwrite`)
  - `get-record` — 查询记录 (`GET` `/v7/low_code_app/app_instance/{app_instance_id}/table/{table_id}/records/{record_id}`; scopes: `kso.low_code_app.read, kso.low_code_app.readwrite`)
  - `get-tables` — 获取应用下的所有数据表 (`POST` `/v7/low_code_app/app_instance/{app_instance_id}/tables`; scopes: `kso.low_code_app.read, kso.low_code_app.readwrite`)
  - `list-records` — 查询记录列表 (`POST` `/v7/low_code_app/app_instance/{app_instance_id}/table/{table_id}/records/search`; scopes: `kso.low_code_app.read, kso.low_code_app.readwrite`)
  - `update-records` — 更新记录 (`POST` `/v7/low_code_app/app_instance/{app_instance_id}/table/{table_id}/records/{record_id}/update`; scopes: `kso.low_code_app.readwrite`)

## Discovering Commands

```bash
wpscli low_code_app --help
wpscli schema low_code_app
```
