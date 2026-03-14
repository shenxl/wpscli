---
name: wps-helper-chat
version: 1.0.0
description: "WPS helper command: chat"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli chat --help"
    auth_types: ["user", "app"]
---

# chat helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 会话与消息助手命令

```bash
wpscli chat <command> [flags]
```

## Commands

### chats

列出当前可见会话

```bash
wpscli chat chats
```

### push

发送文本消息

```bash
wpscli chat push
```

| Arg | Required | Description |
|-----|----------|-------------|
| `chat-id` | yes | 会话 ID |
| `--text` | yes | 消息内容 |

## Examples

```bash
示例：
  wpscli chat chats
  wpscli chat push <chat_id> --text "你好，今天 3 点开会"
```
