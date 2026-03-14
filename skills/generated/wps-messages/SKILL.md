---
name: wps-messages
version: 1.0.0
description: "WPS OpenAPI service: messages"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli messages --help"
    auth_types: ["app"]
---

# messages service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli messages <endpoint> [flags]
```

## API Resources

### messages

  - `create-msg` — 批量发送消息 (`POST` `/v7/messages/batch_create`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `get-message-ids` — 根据三方业务id获取消息id (`POST` `/v7/messages/get_message_ids`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `recall-msg` — 撤回消息 (`POST` `/v7/messages/{message_id}/recall`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `single-create-msg` — 发送消息 (`POST` `/v7/messages/create`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `update-msg` — 更新消息 (`POST` `/v7/messages/{message_id}/update`; scopes: `kso.chat_message.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli messages --help
wpscli schema messages
```
