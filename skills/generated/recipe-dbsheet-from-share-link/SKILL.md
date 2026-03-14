---
name: recipe-dbsheet-from-share-link
version: 1.0.0
description: "从 link_id 解析 file_id 后执行 schema 查询和记录更新。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "productivity"
---

# 从分享链接读取并更新多维表

从 link_id 解析 file_id 后执行 schema 查询和记录更新。

- Services: `links`, `coop_dbsheet`
- Auth sequence: `user -> user`

## Steps

1. 解析分享链接: `wpscli doc resolve-link 'https://365.kdocs.cn/l/<link_id>' --auth-type user`
2. 读取 schema: `wpscli dbsheet schema --file-id <file_id> --auth-type user`
3. 按条件查询: `wpscli dbsheet select --file-id <file_id> --sheet-id <sheet_id> --where "状态 = '进行中'" --auth-type user`
4. 批量更新: `wpscli dbsheet update --file-id <file_id> --sheet-id <sheet_id> --data-json @updates.json --batch-size 100 --auth-type user`
