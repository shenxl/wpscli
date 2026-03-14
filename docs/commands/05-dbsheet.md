# 多维表命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

`dbsheet` SQL-like 命令与 `dbt` 兼容命令。

---

## dbsheet

```bash
wpscli dbsheet --help
```

```text
多维表语义助手（SQL-like + 视图/Webhook/分享视图/表单/仪表盘）

Usage: dbsheet [COMMAND]

Commands:
  schema                   获取多维表 schema（支持 --url 或 --file-id）
  list-sheets              列出所有工作表
  init                     基于 schema 文件初始化多维表结构
  select                   SQL-like 查询记录
  insert                   SQL-like 插入记录
  update                   SQL-like 更新记录
  delete                   SQL-like 删除记录（record-id/record-ids/where 三选一）
  view-list                列出指定工作表视图（内化 dbsheet 视图 API）
  view-get                 获取单个视图详情（内化 dbsheet 视图 API）
  view-create              创建视图（通过结构化入参，不暴露底层路径）
  view-update              更新视图（通过结构化入参，不暴露底层路径）
  view-delete              删除视图（通过结构化入参，不暴露底层路径）
  webhook-list             列出 webhook（内化 dbsheet webhook API）
  webhook-create           创建 webhook（通过结构化入参，不暴露底层路径）
  webhook-delete           删除 webhook（通过结构化入参，不暴露底层路径）
  share-status             查询视图分享状态
  share-enable             开启视图分享链接
  share-disable            关闭视图分享链接
  share-permission-update  更新视图分享权限（通过 --params-json 提供参数）
  form-meta                查询表单元数据
  form-meta-update         更新表单元数据（通过 --params-json 提供参数）
  form-fields              列出表单问题字段
  form-field-update        更新表单问题字段（通过 --params-json 提供参数）
  dashboard-list           列出仪表盘
  dashboard-copy           复制仪表盘
  clean                    清理默认字段和默认空行
  help                     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli dbsheet schema --url "https://365.kdocs.cn/l/xxxx" --user-token
  wpscli dbsheet select --url "https://365.kdocs.cn/l/xxxx" --sheet-id 2 --where "状态 = '进行中'" --fields "状态,负责人" --user-token
  wpscli dbsheet insert --url "https://365.kdocs.cn/l/xxxx" --sheet-id 2 --data-json '[{"标题":"A"}]' --user-token
  wpscli dbsheet view-list --url "https://365.kdocs.cn/l/xxxx" --sheet-id 2 --user-token
  wpscli dbsheet webhook-list --url "https://365.kdocs.cn/l/xxxx" --with-detail --user-token
  wpscli dbsheet share-status --url "https://365.kdocs.cn/l/xxxx" --sheet-id 2 --view-id V1 --user-token
  wpscli dbsheet form-meta --url "https://365.kdocs.cn/l/xxxx" --sheet-id 2 --view-id V1 --user-token
  wpscli dbsheet dashboard-list --url "https://365.kdocs.cn/l/xxxx" --user-token
```

## dbt

```bash
wpscli dbt --help
```

```text
DBSheet 兼容助手（旧版命令）

Usage: dbt [COMMAND]

Commands:
  schema      读取多维表 schema
  list        列出记录
  create      创建记录
  update      更新记录
  delete      删除记录
  import-csv  从 CSV 导入记录
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli dbt schema <file-id>
  wpscli dbt list <file-id> <sheet-id>
  wpscli dbt create <file-id> <sheet-id> --body '{"records":[...]}'
```
