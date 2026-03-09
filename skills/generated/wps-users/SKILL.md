---
name: wps-users
version: 1.0.0
description: "WPS OpenAPI service: users"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli users --help"
---

# users service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli users <endpoint> [flags]
```

## API Resources

### users

  - `batch-disable-user` — 批量禁用用户 (`POST` `/v7/users/batch_disable`; scopes: `kso.contact.readwrite`)
  - `batch-enable-user` — 批量启用用户 (`POST` `/v7/users/batch_enable`; scopes: `kso.contact.readwrite`)
  - `batch-get-user` — 批量查询用户 (`POST` `/v7/users/batch_read`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `batch-get-user-attribute` — 批量获取用户的自定义属性值 (`POST` `/v7/users/custom_attrs/batch_read`; scopes: `kso.user_custom_attr.readwrite`)
  - `batch-update-user-attribute` — 批量更新用户的自定义属性值 (`POST` `/v7/users/custom_attrs/batch_update`; scopes: `kso.user_custom_attr.readwrite`)
  - `batch-update-user-dept` — 批量更新用户所在部门 (`POST` `/v7/users/batch_update_dept`; scopes: `kso.contact.readwrite`)
  - `batch-update-user-order` — 批量修改用户在部门中排序值 (`POST` `/v7/users/batch_update_order`; scopes: `kso.contact.readwrite`)
  - `create-user` — 创建用户 (`POST` `/v7/users/create`; scopes: `kso.contact.readwrite`)
  - `delete-user` — 删除用户 (`POST` `/v7/users/{user_id}/delete`; scopes: `kso.contact.readwrite`)
  - `get-all-user` — 查询企业下所有用户 (`GET` `/v7/users`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-current-user-id` — 获取用户id信息 (`GET` `/v7/users/current_id`; scopes: `kso.user_current_id.read`)
  - `get-email-user` — 根据邮箱获取用户 (`POST` `/v7/users/by_emails`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-ex-user` — 根据ex_user_id获取用户信息 (`POST` `/v7/users/by_ex_user_ids`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-phone-user` — 根据手机号获取用户 (`POST` `/v7/users/by_phones`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-user` — 查询指定用户 (`GET` `/v7/users/{user_id}`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `get-user-dept` — 获取用户所在部门列表 (`GET` `/v7/users/{user_id}/depts`; scopes: `kso.contact.readwrite, kso.contact.read`)
  - `update-user` — 更新用户 (`POST` `/v7/users/{user_id}/update`; scopes: `kso.contact.readwrite`)
  - `user-info` — 获取用户信息 (`GET` `/v7/users/current`; scopes: `kso.user_base.read`)
  - `user-logout` — 用户登出 (`GET` `/v7/users/batch_logout`; scopes: `kso.contact.readwrite, kso.contact.read`)

## Discovering Commands

```bash
wpscli users --help
wpscli schema users
```
