---
name: wps-cookie_im
version: 1.0.0
description: "WPS OpenAPI service: cookie_im"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli cookie_im --help"
    auth_types: ["cookie"]
---

# cookie_im service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli cookie_im <endpoint> [flags]
```

## API Resources

### chats

  - `search-chats-cookie` — 按关键字搜索会话（私有 V7） (`GET` `/v7/chats/search`; scopes: `-`; auth: `cookie-only`)
  - `send-cloud-file-cookie` — 向会话发送云文档消息（私有 V7） (`POST` `/v7/chats/{chat_id}/messages/send_cloud_file`; scopes: `-`; auth: `cookie-only`)

### messages

  - `search-messages-cookie` — 跨会话搜索消息（私有 V7） (`GET` `/v7/messages/search`; scopes: `-`; auth: `cookie-only`)

### recent_chats

  - `list-recent-chats-cookie` — 读取最近会话（私有 V7） (`GET` `/v7/recent_chats`; scopes: `-`; auth: `cookie-only`)

## Discovering Commands

```bash
wpscli cookie_im --help
wpscli schema cookie_im
```
