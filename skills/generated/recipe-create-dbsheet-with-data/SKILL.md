---
name: recipe-create-dbsheet-with-data
version: 1.0.0
description: "创建 dbt 文件、授权用户、建表并批量写入记录。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "productivity"
---

# 创建多维表并初始化数据

创建 dbt 文件、授权用户、建表并批量写入记录。

- Services: `drives`, `files`, `coop_dbsheet`
- Auth sequence: `app -> app -> user -> user`

## Steps

1. 获取应用盘: `wpscli drives list-drives --auth-type app --query allotee_type=app`
2. 创建多维表文件: `wpscli drives create-file --auth-type app --path-param drive_id=<drive_id> --path-param parent_id=0 --body '{"name":"项目管理.dbt","file_type":"file"}'`
3. 授予目标用户可编辑权限: `wpscli files batch-create-file-permission --auth-type app --path-param drive_id=<drive_id> --path-param file_id=<file_id> --body '{...}'`
4. 初始化表结构: `wpscli dbsheet init --file-id <file_id> --schema-file <schema.yaml> --auth-type user`
5. 批量写入记录: `wpscli dbsheet insert --file-id <file_id> --sheet-id <sheet_id> --data-json @records.json --batch-size 100 --auth-type user`

## Caution

多维表写入必须使用 batch 模式，建议 batch_size=100。
