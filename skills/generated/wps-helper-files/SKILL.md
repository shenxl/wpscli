---
name: wps-helper-files
version: 1.0.0
description: "WPS helper command: files"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli files --help"
    auth_types: ["user", "app"]
---

# files helper

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.

- Recommended auth: `user`
- Scope: 应用目录与文件编排（创建、上传、下载、状态管理）

```bash
wpscli files <command> [flags]
```

## Commands

### add-file

在已有应用下新增文件

```bash
wpscli files add-file
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--app` | yes | 应用目录名称 |
| `--file` | yes | 文件名（含扩展名） |
| `--on-name-conflict` | no | 同名冲突策略：fail/rename/replace |
| `--drive-id` | no | 网盘 ID（不传则自动探测） |

### create

创建应用文件（会自动创建目录）

```bash
wpscli files create
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--app` | yes | 应用目录名称 |
| `--file` | yes | 文件名（含扩展名） |
| `--on-name-conflict` | no | 同名冲突策略：fail/rename/replace |
| `--drive-id` | no | 网盘 ID（不传则自动探测） |

### download

下载文件到本地

```bash
wpscli files download
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--drive-id` | no | 网盘 ID |
| `--file-id` | no | 文件 ID（优先） |
| `--app` | no | 应用目录名称（与 --file 搭配） |
| `--file` | no | 文件名（与 --app 搭配） |
| `--output` | no | 本地输出路径（可为目录或文件路径） |
| `--overwrite` | no | 允许覆盖本地已存在文件 |
| `--with-hash` | no | 请求下载信息时附带 hashes |
| `--internal` | no | 优先请求内网下载地址 |
| `--storage-base-domain` | no | 下载域名偏好：wps.cn/kdocs.cn/wps365.com |

### ensure-app

确保应用目录存在

```bash
wpscli files ensure-app
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--app` | yes | 应用目录名称 |
| `--drive-id` | no | 网盘 ID（不传则自动探测） |

### get

查询文件信息

```bash
wpscli files get
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--drive-id` | no | 网盘 ID |
| `--file-id` | no | 文件 ID（优先） |
| `--app` | no | 应用目录名称（与 --file 搭配） |
| `--file` | no | 文件名（与 --app 搭配） |

### list-apps

列出 应用/openclaw 下的应用目录

```bash
wpscli files list-apps
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--drive-id` | no | 网盘 ID（不传则自动探测） |

### list-files

列出某应用下的文件

```bash
wpscli files list-files
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--app` | yes | 应用目录名称 |
| `--drive-id` | no | 网盘 ID（不传则自动探测） |

### state

查看本地状态仓库（registry/log）

```bash
wpscli files state
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--limit` | no | 返回最近操作日志条数 |

### transfer

统一传输视图（带阶段耗时与恢复建议）

```bash
wpscli files transfer
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--mode` | yes | 传输模式：upload/download |
| `--local-file` | no | upload 模式：本地文件路径 |
| `--name` | no | upload 模式：远端文件名（默认本地文件名） |
| `--app` | no | 应用目录（与 --parent-id 或 --file-id 组合） |
| `--parent-id` | no | upload 模式：目标父目录 ID（优先于 --app） |
| `--on-name-conflict` | no | upload 模式：同名冲突策略 |
| `--drive-id` | no | 网盘 ID（不传则自动探测） |
| `--file-id` | no | download 模式：文件 ID（优先） |
| `--file` | no | download 模式：文件名（与 --app 搭配） |
| `--output` | no | download 模式：本地输出路径 |
| `--overwrite` | no | download 模式：允许覆盖本地文件 |
| `--with-hash` | no | download 模式：返回 hash 校验值 |
| `--internal` | no | 优先请求内网传输地址 |
| `--storage-base-domain` | no | download 模式：下载域名偏好 wps.cn/kdocs.cn/wps365.com |

### upload

上传本地文件（请求上传->存储上传->提交完成）

```bash
wpscli files upload
```

| Arg | Required | Description |
|-----|----------|-------------|
| `--auth-type` | no | 鉴权类型：app / user / cookie |
| `--user-token` | no | 快捷方式：等价于 --auth-type user |
| `--dry-run` | no | - |
| `--retry` | no | 网络错误重试次数 |
| `--local-file` | yes | 本地文件路径 |
| `--name` | no | 远端文件名（默认取本地文件名） |
| `--app` | no | 目标应用目录（与 --parent-id 二选一，默认根目录） |
| `--parent-id` | no | 目标父目录 ID（优先于 --app） |
| `--on-name-conflict` | no | 同名冲突策略：fail/rename/overwrite/replace |
| `--internal` | no | 优先请求内网上传地址 |
| `--drive-id` | no | 网盘 ID（不传则自动探测） |

## Examples

```bash
示例：
  wpscli files list-apps --user-token
  wpscli files create --app "Demo" --file "日报.otl" --user-token
  wpscli files list-files --app "Demo" --user-token
```
