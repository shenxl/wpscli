---
name: wps-doc-rw
version: 1.1.0
description: "WPS document reading skill on top of wpscli: resolve share links, read document content, read file metadata, list children, and search files."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli doc --help"
---

# wps-doc-rw

> **PREREQUISITE:** Read `../generated/wps-shared/SKILL.md` for auth and global flags.

`wps-doc-rw` in `wpscli` focuses on end-to-end document reading flow:

1. share URL -> `drive_id/file_id`
2. read doc content (`markdown/plain/kdc`)
3. inspect file metadata
4. list child files
5. keyword search
6. write content by format (`otl` / `dbt`)

## Quick Start

```bash
# 1) Auth
wpscli auth setup --ak <AK> --sk <SK>
wpscli auth login --user --print-url-only
wpscli auth login --user --code <authorization_code>

# 2) Resolve a share link
wpscli doc resolve-link "https://365.kdocs.cn/l/<link_id>" --user-token

# 3) Read markdown content directly from URL
wpscli doc read-doc --url "https://365.kdocs.cn/l/<link_id>" --format markdown --user-token
```

## Commands

### 1) Resolve Share Link

```bash
wpscli doc resolve-link "https://365.kdocs.cn/l/<link_id>" --user-token
```

Output includes:
- `link_id`
- `drive_id`
- `file_id`
- `link_status`

---

### 2) Read Document Content

#### Read by URL (recommended)

```bash
wpscli doc read-doc --url "https://365.kdocs.cn/l/<link_id>" --format markdown --user-token
```

#### Read by IDs

```bash
wpscli doc read-doc --drive-id <drive_id> --file-id <file_id> --format markdown --user-token
```

#### Optional parameters

```bash
# async/auto mode
wpscli doc read-doc --url "https://365.kdocs.cn/l/<link_id>" --mode async --user-token

# query async task result
wpscli doc read-doc --drive-id <drive_id> --file-id <file_id> --mode async --task-id <task_id> --user-token

# include elements
wpscli doc read-doc --url "https://365.kdocs.cn/l/<link_id>" --include-elements para,table,component --user-token
```

`--format` supports: `markdown`, `plain`, `kdc`.

For `.xlsx`/spreadsheet files, `wpscli` proactively routes to sheets APIs first:
- `get-worksheet`
- `get-range-data`

If file type detection misses and `/content` still fails (e.g. `400008018`), the same sheets chain is used as fallback.

For `pdf/ppt/pptx/doc/docx`, `wpscli` automatically prefers `mode=auto` on content extraction and will poll async tasks until completion (within `--max-wait-seconds`).

Tune fallback limits with:
- `--xlsx-max-sheets`
- `--xlsx-max-rows`
- `--xlsx-max-cols`

Tune async polling with:
- `--poll-interval-ms`
- `--max-wait-seconds`

---

### 3) Read File Metadata

```bash
# by URL
wpscli doc file-info --url "https://365.kdocs.cn/l/<link_id>" --user-token

# by IDs
wpscli doc file-info --drive-id <drive_id> --file-id <file_id> --user-token
```

---

### 4) List Child Files

```bash
# by URL, parent_id defaults to resolved file_id
wpscli doc list-files --url "https://365.kdocs.cn/l/<link_id>" --page-size 50 --user-token

# explicit parent folder
wpscli doc list-files --drive-id <drive_id> --parent-id <parent_id> --page-size 100 --user-token
```

---

### 5) Search Files

```bash
wpscli doc search --keyword "周报" --user-token
wpscli doc search --keyword "项目计划" --drive-id <drive_id> --user-token
```

---

### 6) Write Document Content (otl/dbt)

`write-doc` routes by file extension:
- `.otl` -> airpage markdown write
- `.dbt` -> dbsheet records write

#### OTL write (markdown)

```bash
# replace content
wpscli doc write-doc --url "https://365.kdocs.cn/l/<link_id>" --user-token \
  --content "# 标题\n\n正文内容"

# append content
wpscli doc write-doc --url "https://365.kdocs.cn/l/<link_id>" --user-token \
  --write-mode append --content-file ./content.md
```

#### DBT write (records)

```bash
# create records
wpscli doc write-doc --url "https://365.kdocs.cn/l/<link_id>" --user-token \
  --sheet-id 1 --db-action create \
  --records-json '{"records":[{"fields_value":"{\"标题\":\"测试\"}"}]}'

# update records
wpscli doc write-doc --url "https://365.kdocs.cn/l/<link_id>" --user-token \
  --sheet-id 1 --db-action update --records-file ./update_records.json

# delete records
wpscli doc write-doc --url "https://365.kdocs.cn/l/<link_id>" --user-token \
  --sheet-id 1 --db-action delete --records-file ./delete_records.json
```

---

### 7) DBSheet SQL-like（schema/init/select/insert/update/delete）

`wpscli dbsheet` 提供基于 schema 的多维表初始化，以及 SQL-like 的增删改查能力。

#### 基于 schema 初始化

```bash
wpscli dbsheet init \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --schema /path/to/test_table.yaml \
  --sheet-key test \
  --force-recreate \
  --user-token
```

#### 查询（select）

```bash
# 查询全部（分页后本地过滤/投影）
wpscli dbsheet select --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 --limit 50 --offset 0 --user-token

# 带 where + fields
wpscli dbsheet select --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 \
  --where "城市 = '北京' AND 年龄 >= 18" \
  --fields "姓名,城市,年龄" \
  --limit 20 --user-token
```

#### 插入（insert）

```bash
# 单条对象
wpscli dbsheet insert --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 \
  --data-json '{"姓名":"张三","年龄":28,"城市":"深圳"}' \
  --batch-size 100 --user-token

# 批量对象数组
wpscli dbsheet insert --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 \
  --data-file ./insert_rows.json \
  --batch-size 100 --user-token
```

#### 更新（update）

```bash
# 指定 record-id 更新一条
wpscli dbsheet update --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 \
  --record-id "rec_xxx" \
  --data-json '{"城市":"上海"}' \
  --batch-size 100 --user-token

# 批量更新（每条含 id）
wpscli dbsheet update --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 \
  --data-file ./update_rows.json \
  --batch-size 100 --user-token
```

#### 删除（delete）

```bash
# 按 ID 删除
wpscli dbsheet delete --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 --record-ids "rec_a,rec_b" \
  --batch-size 100 --user-token

# 按 where 删除（先查后删）
wpscli dbsheet delete --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 --where "城市 = '广州'" --limit 200 \
  --batch-size 100 --user-token
```

#### 清理默认字段/空行

```bash
wpscli dbsheet clean --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 1 --user-token
```

## Agent Calling Pattern

Use this pattern for stable automation:

```bash
# Discover shape first
wpscli doc --help
wpscli schema drives get-file-content

# Safe preview before write-like calls
wpscli doc read-doc --url "https://365.kdocs.cn/l/<link_id>" --dry-run --user-token
```

## Troubleshooting

- `403 无权限`
  - ensure `--user-token` is used for user-owned files
  - ensure the authenticated user can access the target file
- `invalid share link`
  - expected format: `https://*.kdocs.cn/l/<link_id>`
- `missing --drive-id / --file-id`
  - provide either both IDs or `--url`
