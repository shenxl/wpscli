---
name: wps-devhub
version: 1.0.0
description: "WPS OpenAPI service: devhub"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli devhub --help"
---

# devhub service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli devhub <endpoint> [flags]
```

## API Resources

### devhub

  - `app-detail` — 智能体应用详情查询 (`GET` `/v7/devhub/apps/{app_id}/detail`; scopes: `kso.devhub_app.readwrite, kso.devhub_app.readwrite`)
  - `chat-cancel` — 取消对话 (`GET` `/v7/devhub/apps/{app_id}/chats/cancel`; scopes: `kso.devhub_chat.readwrite`)
  - `session-create` — 创建会话 (`GET` `/v7/devhub/apps/{app_id}/sessions/create`; scopes: `kso.devhub_session.readwrite`)
  - `session-delete` — 删除会话 (`GET` `/v7/devhub/sessions/{session_id}/delete`; scopes: `kso.devhub_session.readwrite`)
  - `session-list` — 查询会话列表 (`GET` `/v7/devhub/apps/{app_id}/sessions/list`; scopes: `kso.devhub_session.readwrite`)
  - `session-message` — 查询会话历史消息 (`GET` `/v7/devhub/apps/{app_id}/sessions/messages`; scopes: `kso.devhub_session.readwrite`)
  - `session-single-name` — 查询单个会话名称 (`GET` `/v7/devhub/sessions/{session_id}/name`; scopes: `kso.devhub_session.readwrite`)

## Discovering Commands

```bash
wpscli devhub --help
wpscli schema devhub
```
