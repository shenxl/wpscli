---
name: wps-groups
version: 1.0.0
description: "WPS OpenAPI service: groups"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli groups --help"
    auth_types: ["app", "user"]
---

# groups service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli groups <endpoint> [flags]
```

## API Resources

### groups

  - `add-member` — 添加用户组成员 (`POST` `/v7/groups/{group_id}/members/create`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `batch-add-member` — 批量添加用户组成员 (`POST` `/v7/groups/{group_id}/members/batch_create`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `batch-delete-member` — 批量删除用户组成员 (`POST` `/v7/groups/{group_id}/members/batch_delete`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `batch-get-member` — 批量获取用户组成员 (`POST` `/v7/groups/{group_id}/members/batch_read`; scopes: `kso.group.readwrite, kso.group.read, kso.group.readwrite, kso.group.read`; auth: `both`)
  - `create-group` — 创建用户组 (`POST` `/v7/groups/create`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `delete-group` — 删除用户组 (`POST` `/v7/groups/{group_id}/delete`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `delete-member` — 删除用户组成员 (`POST` `/v7/groups/{group_id}/members/delete`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `get-group` — 获取用户组 (`GET` `/v7/groups/{group_id}`; scopes: `kso.group.readwrite, kso.group.read, kso.group.readwrite, kso.group.read`; auth: `both`)
  - `get-group-list` — 获取用户组列表 (`GET` `/v7/groups`; scopes: `kso.group.readwrite, kso.group.read, kso.group.readwrite, kso.group.read`; auth: `both`)
  - `get-group-settings` — 查询用户组配置 (`GET` `/v7/groups/{group_id}/settings`; scopes: `kso.group.readwrite, kso.group.read, kso.group.readwrite, kso.group.read`; auth: `both`)
  - `get-member` — 获取用户组成员 (`GET` `/v7/groups/{group_id}/members/read`; scopes: `kso.group.readwrite, kso.group.read, kso.group.readwrite, kso.group.read`; auth: `both`)
  - `get-member-list` — 获取用户组成员列表 (`GET` `/v7/groups/{group_id}/members`; scopes: `kso.group.readwrite, kso.group.read, kso.group.readwrite, kso.group.read`; auth: `both`)
  - `restore-group` — 恢复用户组 (`POST` `/v7/groups/{group_id}/restore`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `search-members` — 搜索用户组成员 (`GET` `/v7/groups/{group_id}/members/search`; scopes: `kso.group.readwrite, kso.group.read, kso.group.readwrite, kso.group.read`; auth: `both`)
  - `update-group` — 更新用户组 (`POST` `/v7/groups/{group_id}/update`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `update-group-owner` — 更新用户组拥有者 (`POST` `/v7/groups/{group_id}/update_owner`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `update-group-settings` — 更新用户组配置 (`POST` `/v7/groups/{group_id}/settings`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)
  - `update-member-role` — 更新用户组成员角色 (`POST` `/v7/groups/{group_id}/members/update_role`; scopes: `kso.group.readwrite, kso.group.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli groups --help
wpscli schema groups
```
