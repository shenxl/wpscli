---
name: wps-chats
version: 1.0.0
description: "WPS OpenAPI service: chats"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli chats --help"
    auth_types: ["app", "user"]
---

# chats service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli chats <endpoint> [flags]
```

## API Resources

### chats

  - `batch-delete-chat-member` — 批量删除群成员 (`POST` `/v7/chats/{chat_id}/members/batch_delete`; scopes: `kso.chat.readwrite`; auth: `both`)
  - `batch-get-chat` — 批量获取会话信息 (`POST` `/v7/chats/batch_get`; scopes: `kso.chat.readwrite, kso.chat.read`; auth: `both`)
  - `batch-get-chat-member` — 批量添加群成员 (`POST` `/v7/chats/{chat_id}/members/batch_create`; scopes: `kso.chat.readwrite`; auth: `both`)
  - `create-chat` — 创建会话（单聊或群聊） (`POST` `/v7/chats/create`; scopes: `kso.chat.readwrite`; auth: `both`)
  - `create-dept-chat` — 创建部门群 (`POST` `/v7/chats/create_dept_chat`; scopes: `kso.chat_dept.readwrite`; auth: `app`)
  - `delete-chat` — 解散群聊 (`POST` `/v7/chats/{chat_id}/delete`; scopes: `kso.chat.readwrite`; auth: `both`)
  - `gen-chat-share-link` — 获取群分享链接 (`GET` `/v7/chats/{chat_id}/share_link`; scopes: `kso.chat.readwrite, kso.chat.read`; auth: `both`)
  - `get-chat-bookmark` — 获取会话书签 (`GET` `/v7/chats/{chat_id}/bookmarks`; scopes: `kso.chat_bookmark.readwrite, kso.chat_bookmark.read`; auth: `both`)
  - `get-chat-info` — 获取会话信息 (`GET` `/v7/chats/{chat_id}`; scopes: `kso.chat.readwrite, kso.chat.read`; auth: `both`)
  - `get-chat-list` — 获取会话列表 (`GET` `/v7/chats`; scopes: `kso.chat.readwrite, kso.chat.read`; auth: `both`)
  - `get-chat-member` — 获取群成员列表 (`GET` `/v7/chats/{chat_id}/members`; scopes: `kso.chat.readwrite, kso.chat.read`; auth: `both`)
  - `get-chat-message` — 获取会话历史消息 (`GET` `/v7/chats/{chat_id}/messages`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `get-chat-unread-count` — 获取用户会话未读数 (`GET` `/v7/chats/unread_count`; scopes: `kso.chat.readwrite, kso.chat.read`; auth: `both`)
  - `get-chats-download` — 获取会话文件下载地址 (`GET` `/v7/chats/{chat_id}/messages/{message_id}/resources/{storage_key}/download`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `get-message` — 获取指定消息的内容 (`GET` `/v7/chats/{chat_id}/messages/{message_id}`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `get-p2p-chat` — 根据user_id获取chat_id (`GET` `/v7/chats/get_p2p_chat`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `is-in-chat` — 判断用户或机器人是否在群里 (`GET` `/v7/chats/{chat_id}/members/is_in_chat`; scopes: `kso.chat.read, kso.chat.readwrite, kso.chat.read, kso.chat.readwrite`; auth: `both`)
  - `normalize-dept-chat` — 部门群转普通群 (`POST` `/v7/chats/normalize_dept_chat`; scopes: `kso.chat_dept.readwrite`; auth: `app`)
  - `reply-msg` — 回复消息 (`POST` `/v7/chats/{chat_id}/messages/{message_id}/reply`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `update-chat-info` — 更新会话信息 (`POST` `/v7/chats/{chat_id}/update`; scopes: `kso.chat.readwrite`; auth: `both`)
  - `update-dept-chat-owner` — 指定部门群群主 (`POST` `/v7/chats/update_dept_chat`; scopes: `kso.chat_dept.readwrite`; auth: `app`)
  - `upload-chats-resource` — 获取会话文件上传地址 (`GET` `/v7/chats/resources/upload`; scopes: `kso.chat_message.readwrite`; auth: `app`)
  - `urgent-msg` — 发送应用内加急 (`GET` `/v7/chats/{chat_id}/messages/{message_id}/urgent_app`; scopes: `kso.chat_message.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli chats --help
wpscli schema chats
```
