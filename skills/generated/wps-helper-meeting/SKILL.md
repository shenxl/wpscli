---
name: wps-helper-meeting
version: 1.0.0
description: "WPS helper command: meeting"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli meeting --help"
    auth_types: ["user", "app"]
---

# meeting helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 会议创建与参会人编排

```bash
wpscli meeting <command> [flags]
```

## Commands

### analyze

读取并分析会议纪要详情

```bash
wpscli meeting analyze
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--minute-id` | yes | 会议纪要 ID |

## Examples

```bash
示例：
  wpscli meeting analyze --minute-id <minute_id>
```
