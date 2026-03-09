---
name: wps-public_mailboxes
version: 1.0.0
description: "WPS OpenAPI service: public_mailboxes"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli public_mailboxes --help"
---

# public_mailboxes service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli public_mailboxes <endpoint> [flags]
```

## API Resources

### public_mailboxes

  - `create-public-mailbox` — 创建公共邮箱 (`GET` `/v7/public_mailboxes`; scopes: `kso.public_mailbox.readwrite`)
  - `create-public-mailbox-alias` — 创建公共邮箱别名 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/aliases`; scopes: `kso.public_mailbox.readwrite`)
  - `create-public-mailbox-members` — 批量添加公共邮箱成员 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/members/batch_create`; scopes: `kso.public_mailbox.readwrite`)
  - `delete-public-mailbox` — 删除公共邮箱 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/delete`; scopes: `kso.public_mailbox.readwrite`)
  - `delete-public-mailbox-alias` — 删除公共邮箱别名 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/delete_alias`; scopes: `kso.public_mailbox.readwrite`)
  - `delete-public-mailbox-members` — 批量删除公共邮箱成员 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/members/batch_delete`; scopes: `kso.public_mailbox.readwrite`)
  - `get-public-mailbox` — 查询指定公共邮箱 (`GET` `/v7/public_mailboxes/{public_mailbox_id}`; scopes: `kso.public_mailbox.read, kso.public_mailbox.readwrite`)
  - `get-public-mailbox-aliases` — 获取指定公共邮箱别名列表 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/aliases`; scopes: `kso.public_mailbox.read, kso.public_mailbox.readwrite`)
  - `get-public-mailbox-member` — 获取公共邮箱指定成员 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/members/{member_id}`; scopes: `kso.public_mailbox.read, kso.public_mailbox.readwrite`)
  - `get-public-mailbox-members` — 获取公共邮箱成员列表 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/members`; scopes: `kso.public_mailbox.read, kso.public_mailbox.readwrite`)
  - `get-public-mailboxes` — 查询所有公共邮箱 (`GET` `/v7/public_mailboxes`; scopes: `kso.public_mailbox.read, kso.public_mailbox.readwrite`)
  - `update-public-mailbox` — 修改公共邮箱 (`GET` `/v7/public_mailboxes/{public_mailbox_id}/update`; scopes: `kso.public_mailbox.readwrite`)

## Discovering Commands

```bash
wpscli public_mailboxes --help
wpscli schema public_mailboxes
```
