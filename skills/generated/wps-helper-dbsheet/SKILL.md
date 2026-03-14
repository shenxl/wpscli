---
name: wps-helper-dbsheet
version: 1.0.0
description: "WPS helper command: dbsheet"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli dbsheet --help"
    auth_types: ["user", "app"]
---

# dbsheet helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 多维表场景命令（schema/select/insert/update/delete）

```bash
wpscli dbsheet <command> [flags]
```

## Commands

### clean

清理默认字段和默认空行

```bash
wpscli dbsheet clean
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### dashboard-copy

复制仪表盘

```bash
wpscli dbsheet dashboard-copy
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--dashboard-id` | yes | 仪表盘 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### dashboard-list

列出仪表盘

```bash
wpscli dbsheet dashboard-list
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### delete

SQL-like 删除记录（record-id/record-ids/where 三选一）

```bash
wpscli dbsheet delete
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--record-id` | no | 单条记录 ID |
| `--record-ids` | no | 多条记录 ID，逗号分隔 |
| `--where` | no | 按条件删除 |
| `--limit` | no | where 删除时最多选中条数 |
| `--batch-size` | no | 批量删除大小 |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### form-field-update

更新表单问题字段（通过 --params-json 提供参数）

```bash
wpscli dbsheet form-field-update
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 表单视图 ID |
| `--field-id` | yes | 字段 ID |
| `--params-json` | no | 更新参数 JSON（对象） |
| `--params-file` | no | 从文件读取更新参数 JSON |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### form-fields

列出表单问题字段

```bash
wpscli dbsheet form-fields
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 表单视图 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### form-meta

查询表单元数据

```bash
wpscli dbsheet form-meta
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 表单视图 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### form-meta-update

更新表单元数据（通过 --params-json 提供参数）

```bash
wpscli dbsheet form-meta-update
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 表单视图 ID |
| `--params-json` | no | 更新参数 JSON（对象） |
| `--params-file` | no | 从文件读取更新参数 JSON |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### init

基于 schema 文件初始化多维表结构

```bash
wpscli dbsheet init
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--schema` | yes | YAML schema 文件路径 |
| `--sheet-key` | no | 仅初始化指定 sheet key |
| `--force-recreate` | no | 强制重建（危险操作） |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### insert

SQL-like 插入记录

```bash
wpscli dbsheet insert
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--data-json` | no | 插入数据 JSON |
| `--data-file` | no | 从文件读取插入数据 JSON |
| `--batch-size` | no | 批量写入大小 |
| `--prefer-id` | no | 优先使用 id 字段映射 |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### list-sheets

列出所有工作表

```bash
wpscli dbsheet list-sheets
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### schema

获取多维表 schema（支持 --url 或 --file-id）

```bash
wpscli dbsheet schema
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### select

SQL-like 查询记录

```bash
wpscli dbsheet select
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--where` | no | 过滤条件，如: 状态 = '进行中' |
| `--fields` | no | 逗号分隔字段列表 |
| `--limit` | no | 最多返回条数 |
| `--offset` | no | 结果偏移量 |
| `--page-size` | no | 分页抓取时的每页大小 |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### share-disable

关闭视图分享链接

```bash
wpscli dbsheet share-disable
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 视图 ID |
| `--share-id` | yes | 分享 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### share-enable

开启视图分享链接

```bash
wpscli dbsheet share-enable
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 视图 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### share-permission-update

更新视图分享权限（通过 --params-json 提供参数）

```bash
wpscli dbsheet share-permission-update
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 视图 ID |
| `--share-id` | yes | 分享 ID |
| `--params-json` | no | 更新参数 JSON（对象） |
| `--params-file` | no | 从文件读取更新参数 JSON |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### share-status

查询视图分享状态

```bash
wpscli dbsheet share-status
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 视图 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### update

SQL-like 更新记录

```bash
wpscli dbsheet update
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--record-id` | no | 单条记录 ID |
| `--data-json` | no | 更新数据 JSON |
| `--data-file` | no | 从文件读取更新数据 JSON |
| `--batch-size` | no | 批量更新大小 |
| `--prefer-id` | no | 优先使用 id 字段映射 |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### view-create

创建视图（通过结构化入参，不暴露底层路径）

```bash
wpscli dbsheet view-create
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--data-json` | no | 视图创建 payload JSON |
| `--data-file` | no | 从文件读取视图创建 payload JSON |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### view-delete

删除视图（通过结构化入参，不暴露底层路径）

```bash
wpscli dbsheet view-delete
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 视图 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### view-get

获取单个视图详情（内化 dbsheet 视图 API）

```bash
wpscli dbsheet view-get
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 视图 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### view-list

列出指定工作表视图（内化 dbsheet 视图 API）

```bash
wpscli dbsheet view-list
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### view-update

更新视图（通过结构化入参，不暴露底层路径）

```bash
wpscli dbsheet view-update
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--sheet-id` | yes | 工作表 ID |
| `--view-id` | yes | 视图 ID |
| `--data-json` | no | 视图更新 payload JSON |
| `--data-file` | no | 从文件读取视图更新 payload JSON |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### webhook-create

创建 webhook（通过结构化入参，不暴露底层路径）

```bash
wpscli dbsheet webhook-create
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--data-json` | no | webhook 创建 payload JSON |
| `--data-file` | no | 从文件读取 webhook 创建 payload JSON |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### webhook-delete

删除 webhook（通过结构化入参，不暴露底层路径）

```bash
wpscli dbsheet webhook-delete
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--hook-id` | yes | hook ID |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

### webhook-list

列出 webhook（内化 dbsheet webhook API）

```bash
wpscli dbsheet webhook-list
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--with-detail` | no | 是否返回规则详情 |
| `--url` | no | 多维表分享链接 |
| `--file-id` | no | 多维表 file_id |
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |

## Examples

```bash
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
