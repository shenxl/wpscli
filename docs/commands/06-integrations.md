# 其他业务助手命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

`calendar/chat/meeting/airpage` 助手命令。

---

## calendar

```bash
wpscli calendar --help
```

```text
日历助手命令

Usage: calendar [COMMAND]

Commands:
  query  查询日历事件
  busy   查询忙闲状态
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli calendar query --calendar-id <id> --start-time 2026-03-01T00:00:00Z --end-time 2026-03-31T23:59:59Z
  wpscli calendar busy --start-time 2026-03-10T09:00:00Z --end-time 2026-03-10T18:00:00Z
```

## chat

```bash
wpscli chat --help
```

```text
会话与消息助手命令

Usage: chat [COMMAND]

Commands:
  chats  列出当前可见会话
  push   发送文本消息
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli chat chats
  wpscli chat push <chat_id> --text "你好，今天 3 点开会"
```

## meeting

```bash
wpscli meeting --help
```

```text
会议纪要助手命令

Usage: meeting [COMMAND]

Commands:
  analyze  读取并分析会议纪要详情
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli meeting analyze --minute-id <minute_id>
```

## airpage

```bash
wpscli airpage --help
```

```text
智能文档（Airpage）助手命令

Usage: airpage [COMMAND]

Commands:
  query  查询文件元信息（用于确认 Airpage 文件）
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli airpage query <file_id>
```
