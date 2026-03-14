---
name: wps-helper-airpage
version: 1.0.0
description: "WPS helper command: airpage"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli airpage --help"
    auth_types: ["user", "app"]
---

# airpage helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 智能文档块读取与写入

```bash
wpscli airpage <command> [flags]
```

## Commands

### query

查询文件元信息（用于确认 Airpage 文件）

```bash
wpscli airpage query
```

| Arg | Required | Description |
|-----|----------|-------------|
| `file-id` | yes | 文件 ID |

## Examples

```bash
示例：
  wpscli airpage query <file_id>
```
