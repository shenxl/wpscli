# wpscli 命令参考（更新版）

本文档给出当前 `wpscli` 的命令面概览。参数细节以自动生成文档为准：

- `docs/commands/README.md`
- `docs/commands/*.md`

---

## 1) 命令分层

`wpscli` 当前有 4 层调用面：

- Layer 1：业务 helper（`doc/files/users/dbsheet/...`）
- Layer 2：动态 API（`wpscli <service> <endpoint> ...`）
- Layer 3：原始 API（`wpscli raw METHOD PATH ...`）
- Layer 4：质量与诊断（`wpscli quality` + `wpscli doctor`）

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
wpscli quality
```

全局参数：

- `--output json|compact|table`（默认 `json`）

---

## 3) 系统命令

### 3.1 `schema`

```bash
wpscli schema <service> [endpoint] --mode raw|invoke
```

用途：查看 descriptor 与执行导向 schema（含 `command_template` / `invoke_template`）。

### 3.2 `catalog`

```bash
wpscli catalog --mode show|service|ai [service]
```

用途：

- `show`：按导航分类看服务
- `service`：按服务平铺
- `ai`：输出机器可读索引（含 auth/required params/template）

### 3.3 `raw`

```bash
wpscli raw GET|POST|PUT|PATCH|DELETE <PATH|URL> \
  --auth-type app|user|cookie
```

### 3.4 `doctor`

```bash
wpscli doctor
```

用途：本地安装、鉴权与质量摘要诊断（含 `quality_probe`）。

### 3.5 `quality`

```bash
wpscli quality
wpscli quality --connectivity-sample 10
wpscli quality --connectivity-sample 10 --connectivity-auth user
```

用途：

- 静态门禁（descriptor 完整性）
- help/schema 一致性门禁
- dry-run 构造门禁
- 可选连通抽样门禁
- 顶层能力域覆盖校验（8 domains）

---

## 4) 业务 helper（Layer 1）

### 4.1 `auth`

```text
auth guide/setup/login/refresh-user/harden/status/logout
```

### 4.2 `doc`

```text
doc resolve-link/read-doc/write-doc/file-info/list-files/search
```

### 4.3 `files`（别名 `app-files`）

```text
files list-apps/ensure-app/create/add-file/create-file/list-files/get/state/upload/download/transfer
```

### 4.4 `users`

```text
users scope/depts/members/user/list/find/sync/cache-status/cache-clear
```

### 4.5 `dbsheet`

```text
dbsheet schema/list-sheets/init/select/insert/update/delete/clean
+ view/webhook/share/form/dashboard 系列语义命令
```

### 4.6 `dbt`

```text
dbt schema/list/create/update/delete/import-csv
```

### 4.7 其他 helper

```text
calendar / chat / meeting / airpage
```

---

## 5) 动态 API（Layer 2）

动态命令形式：

```bash
wpscli <service> <endpoint> \
  [--path-param k=v] [--query k=v] [--header k=v] [--body json]
```

服务总量与端点总量可通过：

```bash
wpscli quality --output compact
# 或
wpscli catalog --mode ai --output compact
```

---

## 6) 推荐排障与验证命令

```bash
wpscli doctor
wpscli quality
wpscli auth status
wpscli schema <service> <endpoint> --mode invoke
```

如果本地命令与源码不一致：

```bash
cargo install --path . --bin wpscli --force
```
