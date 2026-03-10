---
name: wps-users
description: 查询 WPS 组织架构与用户信息。支持权限范围、部门树、部门成员、用户详情，以及本地缓存优先查询与定时刷新。
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
- `wpscli users sync --max-depts 300 --cache-ttl-seconds 21600`
- `wpscli users cache-status`
- `wpscli users cache-clear`

## 行为说明

- `scope` 使用 `/v7/contacts/permissions_scope?scopes=org`
- `sync` 会拉取部门树与成员并写入 `~/.config/wps/skills/wps-users/org_cache.json`
- `find/list/depts/members/user` 默认优先使用缓存；缓存过期自动刷新（可用 `--cache-ttl-seconds` 控制）
- 可用 `--no-cache` 强制远端查询，`--refresh-cache` 先刷新再查
- 所有命令支持：`--auth-type app|user`、`--user-token`、`--dry-run`、`--retry`
