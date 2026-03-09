---
name: wps-depts
version: 1.0.0
description: "WPS OpenAPI service: depts"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli depts --help"
---

# depts service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli depts <endpoint> [flags]
```

## API Resources

### depts

  - `batch-dept-info` — 批量查询指定部门信息 (`POST` `/v7/depts/batch_read`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `batch-get-dept-user` — 批量查询部门下的成员信息 (`POST` `/v7/depts/{dept_id}/members/batch_read`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `create-dept` — 创建部门 (`POST` `/v7/depts/create`; scopes: `kso.contact.readwrite`)
  - `delete-dept` — 删除部门 (`POST` `/v7/depts/{dept_id}/delete`; scopes: `kso.contact.readwrite`)
  - `get-dept-list` — 查询子部门列表 (`GET` `/v7/depts/{dept_id}/children`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-dept-user` — 查询部门下用户列表 (`GET` `/v7/depts/{dept_id}/members`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-ex-dept` — 根据ex_dept_id获取部门信息 (`POST` `/v7/depts/by_ex_dept_ids`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-root-dept` — 获取根部门 (`GET` `/v7/depts/root`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `join-dept` — 将用户加入到部门 (`POST` `/v7/depts/{dept_id}/members/{user_id}/create`; scopes: `kso.contact.readwrite`)
  - `remove-dept` — 将用户移除部门 (`POST` `/v7/depts/{dept_id}/members/{user_id}/delete`; scopes: `kso.contact.readwrite`)
  - `update-dept` — 更新部门 (`POST` `/v7/depts/{dept_id}/update`; scopes: `kso.contact.readwrite`)

## Discovering Commands

```bash
wpscli depts --help
wpscli schema depts
```
