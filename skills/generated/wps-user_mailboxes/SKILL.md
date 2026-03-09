---
name: wps-user_mailboxes
version: 1.0.0
description: "WPS OpenAPI service: user_mailboxes"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli user_mailboxes --help"
---

# user_mailboxes service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli user_mailboxes <endpoint> [flags]
```

## API Resources

### user_mailboxes

  - `create-alias` — 创建别名邮箱 (`GET` `/v7/user_mailboxes/{user_id}/aliases`; scopes: `kso.user_mailbox.readwrite`)
  - `create-user-mailbox` — 创建用户邮箱 (`GET` `/v7/user_mailboxes/{user_id}/create`; scopes: `kso.user_mailbox.readwrite`)
  - `delete-alias` — 删除别名邮箱 (`GET` `/v7/user_mailboxes/{user_id}/delete_alias`; scopes: `kso.user_mailbox.readwrite`)
  - `delete-user-mailbox` — 删除用户邮箱 (`GET` `/v7/user_mailboxes/{user_id}/delete`; scopes: `kso.user_mailbox.readwrite`)
  - `get-user-aliases` — 获取用户所有别名邮箱 (`GET` `/v7/user_mailboxes/{user_id}/aliases`; scopes: `kso.user_mailbox.readwrite, kso.user_mailbox.read`)
  - `get-user-mailbox` — 根据用户ID获取用户邮箱信息 (`GET` `/v7/user_mailboxes/{user_id}`; scopes: `kso.user_mailbox.readwrite, kso.user_mailbox.read`)

## Discovering Commands

```bash
wpscli user_mailboxes --help
wpscli schema user_mailboxes
```
