# wpscli 命令参考（全量）

本文档汇总当前 `wpscli` 已实现的命令入口（截至当前代码）。

---

## 1) 命令分层

`wpscli` 支持三层调用：

- Layer 1：技能/助手命令（`auth/doc/files/users/dbsheet/...`）
- Layer 2：动态 API 命令（`wpscli <service> <endpoint> ...`）
- Layer 3：原始调用（`wpscli raw METHOD PATH ...`）

---

## 2) 顶层命令（`wpscli --help`）

```text
wpscli auth
wpscli schema
wpscli catalog
wpscli raw
wpscli generate-skills
wpscli completions
wpscli ui
wpscli doctor
```

全局参数：

- `--output json|compact|table`（默认 `json`）

---

## 3) 授权命令（`wpscli auth`）

```text
wpscli auth guide
wpscli auth setup
wpscli auth login
wpscli auth refresh-user
wpscli auth harden
wpscli auth status
wpscli auth logout
```

### 3.1 `auth setup`

```bash
wpscli auth setup --ak <AK> --sk <SK> \
  --redirect-uri <URI> \
  --scope <SCOPE> \
  --oauth-server <URL> \
  --oauth-device-endpoint <URL> \
  --oauth-token-endpoint <URL>
```

### 3.2 `auth login`

```bash
wpscli auth login --user --mode local|remote|remote-device
```

常用参数：

- `--code`、`--callback-url`
- `--print-url-only`
- `--no-open`、`--no-local-server`
- `--timeout-seconds`、`--poll-interval-seconds`
- `--user-token`、`--refresh-token`

### 3.3 `auth harden`

```bash
wpscli auth harden
wpscli auth harden --apply
```

功能：

- 巡检旧明文凭据残留
- 巡检/修复配置文件权限（Unix）
- 检查环境变量与 `.env` 中的敏感信息暴露风险

---

## 4) 系统与框架命令

### 4.1 `schema`

```bash
wpscli schema <service> [endpoint]
```

### 4.2 `catalog`

```bash
wpscli catalog --mode show|service [service]
```

### 4.3 `raw`

```bash
wpscli raw GET|POST|PUT|PATCH|DELETE /v7/xxx \
  --query key=value --header key=value --body '{}'
```

### 4.4 `generate-skills`

```bash
wpscli generate-skills --out-dir skills/generated
```

### 4.5 `completions`

```bash
wpscli completions bash|zsh|fish|powershell|elvish
```

### 4.6 `ui`

```bash
wpscli ui [intro|features|framework|setup|config|format|outro|all]
```

补充：`wpscli guide` 等价于 `wpscli ui all`。

### 4.7 `doctor`

```bash
wpscli doctor
```

---

## 5) 业务 helper 命令（Layer 1）

> 说明：这些命令属于技能路由，部分不在顶层 `--help` 中直接显示。

### 5.1 文档助手：`doc`

```text
wpscli doc resolve-link
wpscli doc read-doc
wpscli doc write-doc
wpscli doc file-info
wpscli doc list-files
wpscli doc search
```

常用：

```bash
wpscli doc read-doc --url "<kdocs_url>" --user-token
wpscli doc write-doc --url "<kdocs_url>" --target-format otl --content "# 标题" --user-token
```

### 5.2 应用文件助手：`files`（别名：`app-files`）

```text
wpscli files list-apps
wpscli files ensure-app
wpscli files create
wpscli files add-file         # 别名: create-file
wpscli files list-files
wpscli files get
wpscli files state
```

### 5.3 用户组织助手：`users`

```text
wpscli users scope
wpscli users depts
wpscli users members
wpscli users user
wpscli users list
wpscli users find
wpscli users sync
```

### 5.4 多维表 SQL-like 助手：`dbsheet`

```text
wpscli dbsheet schema
wpscli dbsheet list-sheets
wpscli dbsheet init
wpscli dbsheet select
wpscli dbsheet insert
wpscli dbsheet update
wpscli dbsheet delete
wpscli dbsheet clean
```

详见：`docs/wpscli-dbsheet-guide.md`

### 5.5 兼容助手：`dbt`

```text
wpscli dbt schema
wpscli dbt list
wpscli dbt create
wpscli dbt update
wpscli dbt delete
wpscli dbt import-csv
```

### 5.6 其他 helper

```text
wpscli calendar query
wpscli calendar busy

wpscli chat chats
wpscli chat push

wpscli meeting analyze

wpscli airpage query
```

---

## 6) 动态 API 命令（Layer 2）

动态命令形式：

```bash
wpscli <service> <endpoint> [--path-param k=v] [--query k=v] [--body json]
```

示例：

```bash
wpscli catalog drives
wpscli drives list-files --path-param drive_id=<id> --path-param parent_id=0 --query page_size=5
```

---

## 7) 常用排障命令

```bash
wpscli doctor
wpscli auth status
wpscli auth harden
wpscli auth harden --apply
```

如果本地命令和源码不一致：

```bash
cargo install --path . --bin wpscli --force
```
