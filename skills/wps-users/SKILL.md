---
name: wps-users
description: 查询 WPS 组织架构与用户信息。支持权限范围、部门树、部门成员、用户详情与轻量同步摘要。
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli users --help"
---

# WPS 用户与组织架构

> PREREQUISITE: 推荐使用 `app` 授权（默认），并确保具备 `kso.contact.read` 或 `kso.contact.readwrite`。

## 核心命令

- `wpscli users scope`
- `wpscli users depts --dept-id root`
- `wpscli users members --dept-id <DEPT_ID>`
- `wpscli users user --user-id <USER_ID> --with-dept true`
- `wpscli users list --keyword "张三"`
- `wpscli users find --name "李"`
- `wpscli users sync --max-depts 200`

## 行为说明

- `scope` 使用 `/v7/contacts/permissions_scope?scopes=org`
- `sync` 为轻量同步摘要：遍历权限范围内部门并采样成员，适合快速体检
- 所有命令支持：`--auth-type app|user`、`--user-token`、`--dry-run`、`--retry`
