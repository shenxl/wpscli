---
name: wps-helper-dbt
version: 1.0.0
description: "WPS helper command: dbt"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli dbt --help"
    auth_types: ["user", "app"]
---

# dbt helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 多维表批量与结构化写入工具

```bash
wpscli dbt <command> [flags]
```

## Commands

### create

创建记录

```bash
wpscli dbt create
```

| Arg | Required | Description |
|-----|----------|-------------|
| `file-id` | yes | 多维表 file_id |
| `sheet-id` | yes | 工作表 sheet_id |
| `--body` | yes | 请求体 JSON 字符串 |

### delete

删除记录

```bash
wpscli dbt delete
```

| Arg | Required | Description |
|-----|----------|-------------|
| `file-id` | yes | 多维表 file_id |
| `sheet-id` | yes | 工作表 sheet_id |
| `--body` | yes | 请求体 JSON 字符串 |

### import-csv

从 CSV 导入记录

```bash
wpscli dbt import-csv
```

| Arg | Required | Description |
|-----|----------|-------------|
| `file-id` | yes | 多维表 file_id |
| `sheet-id` | yes | 工作表 sheet_id |
| `--csv` | yes | CSV 文件路径 |

### list

列出记录

```bash
wpscli dbt list
```

| Arg | Required | Description |
|-----|----------|-------------|
| `file-id` | yes | 多维表 file_id |
| `sheet-id` | yes | 工作表 sheet_id |

### schema

读取多维表 schema

```bash
wpscli dbt schema
```

| Arg | Required | Description |
|-----|----------|-------------|
| `file-id` | yes | 多维表 file_id |

### update

更新记录

```bash
wpscli dbt update
```

| Arg | Required | Description |
|-----|----------|-------------|
| `file-id` | yes | 多维表 file_id |
| `sheet-id` | yes | 工作表 sheet_id |
| `--body` | yes | 请求体 JSON 字符串 |

## Examples

```bash
示例：
  wpscli dbt schema <file-id>
  wpscli dbt list <file-id> <sheet-id>
  wpscli dbt create <file-id> <sheet-id> --body '{"records":[...]}'
```
