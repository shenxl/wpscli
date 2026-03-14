---
name: wps-sse
version: 1.0.0
description: "WPS OpenAPI service: sse"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli sse --help"
    auth_types: ["app", "user"]
---

# sse service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli sse <endpoint> [flags]
```

## API Resources

### sse

  - `chat-create` — 开始对话 (`GET` `/v7/sse/devhub/apps/{app_id}/chats/create`; scopes: `kso.devhub_chat.readwrite`; auth: `user`)
  - `search-gpt` — 团队文档智能问答 (`POST` `/v7/sse/aidocs/search/gpt`; scopes: `kso.aidocs.readwrite, kso.aidocs.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli sse --help
wpscli schema sse
```
