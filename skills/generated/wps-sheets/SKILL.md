---
name: wps-sheets
version: 1.0.0
description: "WPS OpenAPI service: sheets"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli sheets --help"
    auth_types: ["app", "user"]
---

# sheets service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli sheets <endpoint> [flags]
```

## API Resources

### sheets

  - `add-protection` — （工作表）创建区域权限信息 (`GET` `/v7/sheets/{file_id}/protection_ranges`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `copy-worksheet` — 复制工作表 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/copy`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `create-datavalidation` — 创建数据校验 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/data_validations`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `create-row-data` — （工作表）创建行 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/rows`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `create-worksheet` — 创建工作表 (`GET` `/v7/sheets/{file_id}/worksheets`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `delete-datavalidation` — 删除数据校验 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/data_validations/batch_delete`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `delete-filters` — 删除筛选 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/filters/delete`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `delete-protection` — （工作表）删除区域权限 (`GET` `/v7/sheets/{file_id}/protection_ranges/batch_delete`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `delete-range-data` — 删除单元格选区数据 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/range_data/batch_delete`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `delete-worksheet` — 删除工作表 (`GET` `/v7/sheets/{file_id}/worksheets/batch_delete`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `find-range-data` — 查找单元格 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/range_data/find`; scopes: `kso.sheets.readwrite, kso.sheets.read, kso.sheets.readwrite, kso.sheets.read`; auth: `both`)
  - `get-datavalidation` — 获取数据校验 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/data_validations`; scopes: `kso.sheets.readwrite, kso.sheets.read, kso.sheets.readwrite, kso.sheets.read`; auth: `both`)
  - `get-filters` — 获取筛选 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/filters`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `get-protection` — 获取区域权限列表 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/protection_ranges`; scopes: `kso.sheets.readwrite, kso.sheets.read, kso.sheets.readwrite, kso.sheets.read`; auth: `both`)
  - `get-range-data` — 获取单元格选区数据 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/range_data`; scopes: `kso.sheets.readwrite, kso.sheets.read, kso.sheets.readwrite, kso.sheets.read`; auth: `both`)
  - `get-worksheet` — 获取Sheet列表信息 (`GET` `/v7/sheets/{file_id}/worksheets`; scopes: `kso.sheets.readwrite, kso.sheets.read, kso.sheets.readwrite, kso.sheets.read`; auth: `both`)
  - `open-filters` — 开启筛选 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/filters`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `update-datavalidation` — 更新数据校验 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/data_validations/batch_update`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `update-filters` — 更新筛选 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/filters/update`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `update-protection` — （工作表）修改区域权限 (`GET` `/v7/sheets/{file_id}/protection_ranges/batch_update`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `update-range-data` — 更新单元格选区数据 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/range_data/batch_update`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)
  - `update-worksheet` — 更新工作表 (`GET` `/v7/sheets/{file_id}/worksheets/{worksheet_id}/update`; scopes: `kso.sheets.readwrite, kso.sheets.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli sheets --help
wpscli schema sheets
```
