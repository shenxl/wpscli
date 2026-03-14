---
name: recipe-sync-org-to-dbsheet
version: 1.0.0
description: "同步部门和成员并批量落库到多维表。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "hr"
---

# 组织架构同步到多维表

同步部门和成员并批量落库到多维表。

- Services: `users`, `depts`, `coop_dbsheet`, `files`
- Auth sequence: `app -> app -> user`

## Steps

1. 同步部门: `wpscli users depts --auth-type app`
2. 同步成员: `wpscli users members --dept-id root --recursive true --auth-type app`
3. 创建/定位目标表: `wpscli files ensure-app --app 组织同步 --auth-type user`
4. 批量写入成员: `wpscli dbsheet insert --file-id <file_id> --sheet-id <sheet_id> --data-json @members.json --batch-size 100 --auth-type user`

## Caution

跨 auth 切换流程中，必须先完成 app 查询再执行 user 写入。
