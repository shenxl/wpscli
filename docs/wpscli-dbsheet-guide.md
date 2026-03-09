# wpscli dbsheet 命令指南

本文档用于快速上手 `wpscli dbsheet`，覆盖以下场景：

- 基于 schema 初始化多维表
- SQL-like 查询（`where` / `fields` / `limit`）
- 增删改查（insert / update / delete）
- 清理默认字段和空行
- 常见错误排查

---

## 1. 前置条件

先确认你已安装并登录：

```bash
wpscli auth status
```

建议状态满足：

- `has_user_token: true`
- `has_refresh_token: true`
- `auto_refresh_ready: true`

如未登录，执行：

```bash
wpscli auth login --user --mode local
```

---

## 2. dbsheet 能力总览

查看命令入口：

```bash
wpscli dbsheet --help
```

当前子命令：

- `schema`：读取多维表结构
- `list-sheets`：列出工作表
- `init`：按 YAML schema 初始化工作表
- `select`：SQL-like 查询
- `insert`：新增记录（支持批量）
- `update`：更新记录（支持批量）
- `delete`：删除记录（按 id 或 where）
- `clean`：清理默认字段与默认空行

---

## 3. 先探测结构（推荐）

先看表结构和 `sheet_id`：

```bash
wpscli dbsheet schema --url "https://365.kdocs.cn/l/<link_id>" --user-token
wpscli dbsheet list-sheets --url "https://365.kdocs.cn/l/<link_id>" --user-token
```

---

## 4. 基于 schema 初始化（init）

示例 schema：`wpsskill/wps-doc-rw/schemas/test_table.yaml`

```bash
wpscli dbsheet init \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --schema /path/to/test_table.yaml \
  --sheet-key test \
  --force-recreate \
  --user-token
```

说明：

- `--sheet-key`：YAML `sheets` 下的键，不传时默认第一个
- `--force-recreate`：发现同名 sheet 时先删再建
- `auto_clean: true` 时会自动执行默认字段/空行清理

---

## 5. SQL-like 查询（select）

### 5.1 查询全部

```bash
wpscli dbsheet select \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --limit 50 \
  --offset 0 \
  --user-token
```

### 5.2 条件查询

```bash
wpscli dbsheet select \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --where "热点标题 = 'AI 周报' AND 来源 = '公众号'" \
  --fields "热点标题,来源,内容摘要,备注" \
  --limit 20 \
  --user-token
```

`where` 支持：

- 比较：`=`, `!=`, `>`, `<`, `>=`, `<=`
- 逻辑：`AND`, `OR`
- 集合：`IN ('A','B')`
- 模糊：`LIKE '%关键词%'`

---

## 6. 新增记录（insert）

### 6.1 单条

```bash
wpscli dbsheet insert \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --data-json '{"热点标题":"测试标题","来源":"wpscli","内容摘要":"test","备注":"insert"}' \
  --batch-size 100 \
  --user-token
```

### 6.2 批量（推荐）

```bash
wpscli dbsheet insert \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --data-file ./insert_rows.json \
  --batch-size 100 \
  --user-token
```

`insert_rows.json` 示例：

```json
[
  {"热点标题":"A","来源":"x"},
  {"热点标题":"B","来源":"y"}
]
```

> 建议批量写入（`batch-size` 默认 100），可显著降低 API 调用次数并提升稳定性。

---

## 7. 更新记录（update）

### 7.1 指定 record-id 更新单条

```bash
wpscli dbsheet update \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --record-id "rec_xxx" \
  --data-json '{"写作状态":"处理中","备注":"updated-by-wpscli"}' \
  --batch-size 100 \
  --user-token
```

### 7.2 批量更新（每条带 id）

```bash
wpscli dbsheet update \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --data-file ./update_rows.json \
  --batch-size 100 \
  --user-token
```

`update_rows.json` 示例：

```json
[
  {"id":"rec_a","写作状态":"已完成"},
  {"id":"rec_b","备注":"人工复核"}
]
```

---

## 8. 删除记录（delete）

### 8.1 按 id 删除

```bash
wpscli dbsheet delete \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --record-ids "rec_a,rec_b" \
  --batch-size 100 \
  --user-token
```

### 8.2 按 where 删除

```bash
wpscli dbsheet delete \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --where "来源 = 'wpscli-test'" \
  --limit 200 \
  --batch-size 100 \
  --user-token
```

---

## 9. 清理默认项（clean）

```bash
wpscli dbsheet clean \
  --url "https://365.kdocs.cn/l/<link_id>" \
  --sheet-id 2 \
  --user-token
```

清理内容：

- 默认字段：`名称`、`数量`、`日期`、`状态`
- 默认空行：仅保留业务有效记录

---

## 10. 一套安全 CRUD 验证模板

建议每次联调都按以下顺序，保证可回滚：

1. `insert` 一条带唯一标记的数据（例如 `TEST_<timestamp>`）
2. `select --where` 验证命中
3. `update` 修改指定字段
4. `select --where` 复查更新结果
5. `delete` 按 `record-id` 回滚
6. `select --where` 确认为 0 条

---

## 11. 常见问题排查

### Q0: `unrecognized subcommand 'dbsheet'`

说明当前 `wpscli` 二进制是旧版本，尚未包含 `dbsheet` helper。

执行升级安装：

```bash
cd /Users/shen/openclaw_src/googlews/wps-cli
cargo install --path . --bin wpscli --force
wpscli dbsheet --help
```

### Q1: `403` / 权限不足

- 确认使用 `--user-token`
- 确认当前账号对该多维表有编辑权限

### Q2: `auth_error` / token 相关错误

- 先看：`wpscli auth status`
- 立即刷新：`wpscli auth refresh-user`
- 仍失败则重新登录：`wpscli auth login --user --mode local`

### Q3: `where` 不生效

- 检查字段名是否与 schema 完全一致（含中文）
- 先不带 `--fields` 查询全字段定位问题

### Q4: 批量写入超时

- 调低 `--batch-size`（如 100 -> 50）
- 增加 `--retry`（例如 `--retry 2`）

---

## 12. 推荐实践

- 生产写入前先跑 `--dry-run`（非 where 场景）
- 任何大批量写操作都使用批量模式，不逐条调用
- 重要写入操作先 `select` 快照，再执行 `update/delete`
- 保持字段命名稳定，避免 schema 漂移导致 where/update 失效

