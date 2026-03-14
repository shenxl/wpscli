---
name: wps-helper-doc
version: 1.0.0
description: "WPS helper command: doc"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli doc --help"
    auth_types: ["user", "app"]
---

# doc helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 文档统一读写（分享链接解析、读取、写入、搜索）

```bash
wpscli doc <command> [flags]
```

## Commands

### file-info

读取文件元信息

```bash
wpscli doc file-info
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--url` | no | 文档分享链接 |
| `--drive-id` | no | 网盘 ID |
| `--file-id` | no | 文件 ID |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | - |

### list-files

列出子文件/子目录

```bash
wpscli doc list-files
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--url` | no | 分享链接（默认 parent=file_id） |
| `--drive-id` | no | 网盘 ID |
| `--parent-id` | no | 父目录 ID |
| `--page-size` | no | 每页数量 |
| `--page-token` | no | 翻页游标 |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | - |

### read-doc

读取文档内容（支持 URL 或 drive/file_id）

```bash
wpscli doc read-doc
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--url` | no | 文档分享链接 |
| `--drive-id` | no | 网盘 ID（与 --file-id 搭配） |
| `--file-id` | no | 文件 ID |
| `--format` | no | 输出格式（dbt 会路由到 wpscli dbsheet SQL-like 查询模块） |
| `--mode` | no | 读取模式：sync/async/auto |
| `--task-id` | no | 异步任务 ID（续跑时使用） |
| `--include-elements` | no | 元素过滤，逗号分隔：para,table,component,textbox,all |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | - |
| `--poll-interval-ms` | no | 异步任务轮询间隔（毫秒） |
| `--max-wait-seconds` | no | 异步任务最大等待时长（秒） |
| `--xlsx-max-sheets` | no | xlsx 最多读取工作表数量（兼容参数，等价于 --xlsx-sheet-head） |
| `--xlsx-max-rows` | no | xlsx 每个工作表最多读取行数（兼容参数，等价于 --xlsx-row-head） |
| `--xlsx-max-cols` | no | xlsx 每个工作表最多读取列数（兼容参数，等价于 --xlsx-col-head） |
| `--xlsx-sheet-offset` | no | xlsx 读取工作表偏移量（从第几个工作表开始） |
| `--xlsx-sheet-head` | no | xlsx 读取工作表数量上限（优先于 --xlsx-max-sheets） |
| `--xlsx-row-offset` | no | xlsx 每个工作表行偏移量（从 active_area 的第几行开始） |
| `--xlsx-row-head` | no | xlsx 每个工作表读取行数上限（优先于 --xlsx-max-rows） |
| `--xlsx-col-offset` | no | xlsx 每个工作表列偏移量（从 active_area 的第几列开始） |
| `--xlsx-col-head` | no | xlsx 每个工作表读取列数上限（优先于 --xlsx-max-cols） |
| `--output-file` | no | 将 read-doc 结果写入指定 JSON 文件（大数据推荐） |
| `--export-parquet` | no | 将表格结果导出为 Parquet（可选路径；不传值则写入系统临时目录） |
| `--output-stdout` | no | 配合 --output-file 使用：写文件后仍输出完整结果到 stdout |
| `--dbsheet-sheet-id` | no | dbt 读取时指定 sheet_id（可选） |
| `--dbsheet-where` | no | dbt 读取过滤条件（SQL-like） |
| `--dbsheet-fields` | no | dbt 读取字段投影，逗号分隔 |
| `--dbsheet-limit` | no | dbt 读取返回条数上限 |
| `--dbsheet-offset` | no | dbt 读取偏移量 |

### resolve-link

解析分享链接，获取 drive_id/file_id

```bash
wpscli doc resolve-link
```

| Arg | Required | Description |
|-----|----------|-------------|
| `url` | yes | - |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | - |

### search

按关键字搜索文件

```bash
wpscli doc search
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--keyword` | yes | - |
| `--drive-id` | no | - |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | - |

### write-doc

按文件类型写入内容（otl/dbt/xlsx）

```bash
wpscli doc write-doc
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--url` | no | 文档分享链接 |
| `--drive-id` | no | 网盘 ID（与 --file-id 搭配） |
| `--file-id` | no | 文件 ID |
| `--target-format` | no | 目标格式（默认自动识别） |
| `--format` | no | target-format 的兼容别名 |
| `--write-mode` | no | OTL 写入模式：replace 或 append |
| `--content` | no | OTL 写入内容（Markdown） |
| `--content-file` | no | 从文件读取 OTL Markdown 内容 |
| `--db-action` | no | DBT 操作类型 |
| `--sheet-id` | no | DBT 必填：工作表 ID |
| `--records-json` | no | DBT 记录 JSON 字符串 |
| `--records-file` | no | 从文件读取 DBT 记录 JSON |
| `--record-id` | no | DBT update 单条记录 ID |
| `--db-batch-size` | no | DBT 批量写入大小 |
| `--db-prefer-id` | no | DBT 写入优先使用 id 字段映射 |
| `--xlsx-sheet-id` | no | XLSX 写入目标 worksheet_id（可选） |
| `--xlsx-row-from` | no | XLSX 写入起始行（默认 0；append 模式自动追加） |
| `--xlsx-col-from` | no | XLSX 写入起始列（默认取工作表 active_area.col_from） |
| `--xlsx-write-mode` | no | XLSX 写入模式：replace/append |
| `--xlsx-values-json` | no | XLSX 写入二维数组 JSON |
| `--xlsx-values-file` | no | 从文件读取 XLSX 写入 JSON |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | - |

## Examples

```bash
示例：
  wpscli doc read-doc --url "https://365.kdocs.cn/l/xxxx" --user-token
  wpscli doc write-doc --url "https://365.kdocs.cn/l/xxxx" --target-format otl --content "# 标题" --user-token
  wpscli doc write-doc --url "https://365.kdocs.cn/l/xxxx" --target-format xlsx --xlsx-values-json '[["标题","值"],["A",1]]' --user-token
```
