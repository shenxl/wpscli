---
name: wps-companies
version: 1.0.0
description: "WPS OpenAPI service: companies"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli companies --help"
---

# companies service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli companies <endpoint> [flags]
```

## API Resources

### companies

  - `create-attr` — 新增自定义用户属性 (`POST` `/v7/companies/user_custom_attrs/batch_create`; scopes: `kso.user_custom_attr.readwrite`)
  - `delete-attr` — 删除自定义用户属性 (`POST` `/v7/companies/user_custom_attrs/batch_delete`; scopes: `kso.user_custom_attr.readwrite`)
  - `get-attr` — 获取自定义用户属性 (`GET` `/v7/companies/user_custom_attrs/batch_read`; scopes: `kso.user_custom_attr.readwrite`)
  - `get-company-info` — 查询企业信息 (`GET` `/v7/companies/current`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `update-attr` — 修改自定义用户属性 (`POST` `/v7/companies/user_custom_attrs/batch_update`; scopes: `kso.user_custom_attr.readwrite`)

## Discovering Commands

```bash
wpscli companies --help
wpscli schema companies
```
