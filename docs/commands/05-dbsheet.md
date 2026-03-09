# 多维表命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

`dbsheet` SQL-like 命令与 `dbt` 兼容命令。

---

## dbsheet

```bash
wpscli dbsheet --help
```

```text
多维表 SQL-like 助手（schema/init/select/insert/update/delete）

Usage: dbsheet [COMMAND]

Commands:
  schema       获取多维表 schema（支持 --url 或 --file-id）
  list-sheets  列出所有工作表
  init         基于 schema 文件初始化多维表结构
  select       SQL-like 查询记录
  insert       SQL-like 插入记录
  update       SQL-like 更新记录
  delete       SQL-like 删除记录（record-id/record-ids/where 三选一）
  clean        清理默认字段和默认空行
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli dbsheet schema --url "https://365.kdocs.cn/l/xxxx" --user-token
  wpscli dbsheet select --url "https://365.kdocs.cn/l/xxxx" --sheet-id 2 --where "状态 = '进行中'" --fields "状态,负责人" --user-token
  wpscli dbsheet insert --url "https://365.kdocs.cn/l/xxxx" --sheet-id 2 --data-json '[{"标题":"A"}]' --user-token
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
