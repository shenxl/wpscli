---
name: wps-helper-calendar
version: 1.0.0
description: "WPS helper command: calendar"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli calendar --help"
    auth_types: ["user", "app"]
---

# calendar helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 日历查询、创建与忙闲分析

```bash
wpscli calendar <command> [flags]
```

## Commands

### busy

查询忙闲状态

```bash
wpscli calendar busy
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--start-time` | yes | 开始时间（ISO8601） |
| `--end-time` | yes | 结束时间（ISO8601） |

### query

查询日历事件

```bash
wpscli calendar query
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--calendar-id` | yes | 日历 ID |
| `--start-time` | no | 开始时间（ISO8601） |
| `--end-time` | no | 结束时间（ISO8601） |

## Examples

```bash
示例：
  wpscli calendar query --calendar-id <id> --start-time 2026-03-01T00:00:00Z --end-time 2026-03-31T23:59:59Z
  wpscli calendar busy --start-time 2026-03-10T09:00:00Z --end-time 2026-03-10T18:00:00Z
```
