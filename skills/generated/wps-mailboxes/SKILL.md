---
name: wps-mailboxes
version: 1.0.0
description: "WPS OpenAPI service: mailboxes"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli mailboxes --help"
    auth_types: ["user"]
---

# mailboxes service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli mailboxes <endpoint> [flags]
```

## API Resources

### mailboxes

  - `create-draft` — 创建草稿 (`GET` `/v7/mailboxes/{mailbox_id}/messages/create`; scopes: `kso.mail.readwrite`; auth: `user`)
  - `get-folder-list` — 获取目录列表 (`GET` `/v7/mailboxes/{mailbox_id}/folders`; scopes: `kso.mailbox.readwrite, kso.mailbox.read`; auth: `user`)
  - `get-folder-mail-list` — 获取特定目录下的邮件列表 (`GET` `/v7/mailboxes/{mailbox_id}/folders/{folder_id}/messages`; scopes: `kso.mail.readwrite, kso.mail.read`; auth: `user`)
  - `get-mail` — 获取指定邮件 (`GET` `/v7/mailboxes/{mailbox_id}/folders/{folder_id}/messages/{message_id}`; scopes: `kso.mail.readwrite, kso.mail.read`; auth: `user`)
  - `get-mailboxes` — 获取邮箱列表 (`GET` `/v7/mailboxes`; scopes: `kso.mailbox.readwrite, kso.mailbox.read`; auth: `user`)
  - `get-sub-folder-list` — 获取子目录列表 (`GET` `/v7/mailboxes/{mailbox_id}/folders/{folder_id}/children`; scopes: `kso.mailbox.readwrite, kso.mailbox.read`; auth: `user`)
  - `send-draft` — 发送草稿 (`GET` `/v7/mailboxes/{mailbox_id}/messages/{message_id}/send`; scopes: `kso.mail.readwrite`; auth: `user`)

## Discovering Commands

```bash
wpscli mailboxes --help
wpscli schema mailboxes
```
