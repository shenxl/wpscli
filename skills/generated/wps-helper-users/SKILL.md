---
name: wps-helper-users
version: 1.0.0
description: "WPS helper command: users"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli users --help"
    auth_types: ["app"]
---

# users helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `app`
- Scope: 组织通讯录同步与查询（强制 app token）

```bash
wpscli users <command> [flags]
```

## Commands

### cache-clear

清空本地 users 缓存

```bash
wpscli users cache-clear
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### cache-status

查看本地 users 缓存状态

```bash
wpscli users cache-status
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### depts

列出指定部门的子部门（优先缓存）

```bash
wpscli users depts
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--no-cache` | no | 不使用本地缓存，直接请求远端 |
| `--refresh-cache` | no | 先强制刷新缓存再查询 |
| `--cache-ttl-seconds` | no | 缓存有效期（秒） |
| `--dept-id` | no | 部门 ID，默认 root |
| `--page-size` | no | 每页数量 |
| `--page-token` | no | 翻页游标 |

### find

按姓名关键字搜索用户（优先缓存）

```bash
wpscli users find
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--no-cache` | no | 不使用本地缓存，直接请求远端 |
| `--refresh-cache` | no | 先强制刷新缓存再查询 |
| `--cache-ttl-seconds` | no | 缓存有效期（秒） |
| `--name` | yes | 姓名关键字 |
| `--page-size` | no | 每页数量 |
| `--page-token` | no | 翻页游标 |

### list

按条件查询用户列表（优先缓存）

```bash
wpscli users list
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--no-cache` | no | 不使用本地缓存，直接请求远端 |
| `--refresh-cache` | no | 先强制刷新缓存再查询 |
| `--cache-ttl-seconds` | no | 缓存有效期（秒） |
| `--keyword` | no | 姓名/邮箱关键字 |
| `--page-size` | no | 每页数量 |
| `--page-token` | no | 翻页游标 |

### members

列出部门成员（优先缓存）

```bash
wpscli users members
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--no-cache` | no | 不使用本地缓存，直接请求远端 |
| `--refresh-cache` | no | 先强制刷新缓存再查询 |
| `--cache-ttl-seconds` | no | 缓存有效期（秒） |
| `--dept-id` | yes | 部门 ID |
| `--status` | no | 成员状态，逗号分隔 |
| `--recursive` | no | 是否递归子部门 |
| `--with-user-detail` | no | 是否返回用户详细信息 |
| `--page-size` | no | 每页数量 |
| `--page-token` | no | 翻页游标 |

### scope

查看通讯录权限范围（org）

```bash
wpscli users scope
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### sync

同步并刷新用户/部门缓存

```bash
wpscli users sync
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--max-depts` | no | 最多拉取部门数量 |
| `--cache-ttl-seconds` | no | 缓存有效期（秒） |

### user

查询指定用户详情（优先缓存）

```bash
wpscli users user
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--no-cache` | no | 不使用本地缓存，直接请求远端 |
| `--refresh-cache` | no | 先强制刷新缓存再查询 |
| `--cache-ttl-seconds` | no | 缓存有效期（秒） |
| `--user-id` | yes | 用户 ID |
| `--with-dept` | no | 是否携带部门信息 |

## Examples

```bash
示例：
  wpscli users sync --auth-type app --max-depts 300
  wpscli users cache-status
  wpscli users find --name 张三 --auth-type app
  wpscli users members --dept-id root --recursive true --auth-type app
```
