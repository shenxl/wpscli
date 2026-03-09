# wpscli

WPS OpenAPI CLI for humans and AI agents.

`wpscli` 目标是把 WPS OpenAPI 统一为“人类可用 + 大模型可调用”的命令行接口，支持：

- 高层业务技能命令（`doc/files/users/dbsheet`）
- 描述符驱动的动态 API 命令（覆盖广）
- `raw` 直连 API（最灵活）

---

## 当前实现状态（2026-03）

### 已完成能力

- **认证体系**
  - `auth setup / login / refresh-user / status / logout / guide`
  - 本地回调登录、远程粘贴回调登录、设备码登录
  - user token 自动刷新（过期时间与 401/invalid_token 场景）
- **安全存储**
  - 凭据与 token 改为加密落盘：`credentials.enc` / `token_cache.enc`
  - 加密：AES-256-GCM
  - 密钥：优先 OS Keyring，失败时回退到 `~/.config/wps/.encryption_key`
  - 自动迁移旧明文 `credentials.json/token_cache.json`
  - 新增 `wpscli auth harden`（巡检/加固）
- **文档能力（doc）**
  - `read-doc`：按文件格式路由（xlsx/sheets、pdf/doc/content 优先、异步轮询等）
  - `write-doc`：支持 OTL / DBT 写入链路
- **多维表能力（dbsheet）**
  - SQL-like：`schema/list-sheets/init/select/insert/update/delete/clean`
  - 支持 YAML schema 初始化与 where 表达式
  - 写入路径采用批量处理，减少 API 次数
- **业务技能产品化**
  - `files` / `users` helper
  - `skill_runtime`（scope preflight、state store、operation log）
  - `SKILL.md` 执行契约化与 golden cases
- **可观测与排障**
  - `wpscli doctor`（二进制新旧、功能注册、鉴权就绪度）

---

## 安装

在本目录执行：

```bash
cargo install --path . --bin wpscli --force
```

安装后验证：

```bash
wpscli --help
wpscli doctor
```

---

## 快速开始

### 1) 认证配置

```bash
wpscli auth setup --ak <AK> --sk <SK>
```

或交互引导：

```bash
wpscli auth guide
```

### 2) 用户登录

本地模式（推荐）：

```bash
wpscli auth login --user --mode local
```

远程模式：

```bash
wpscli auth login --user --mode remote
```

设备码模式：

```bash
wpscli auth login --user --mode remote-device
```

### 3) 状态与安全检查

```bash
wpscli auth status
wpscli auth harden
wpscli auth harden --apply
wpscli doctor
```

---

## 核心命令示例

### 文档读取

```bash
wpscli doc read-doc --url "https://365.kdocs.cn/l/xxxx" --user-token
```

### 多维表查询

```bash
wpscli dbsheet select \
  --url "https://365.kdocs.cn/l/xxxx" \
  --sheet-id 2 \
  --where "状态 = '进行中'" \
  --fields "状态,负责人" \
  --limit 20 \
  --user-token
```

### 动态 API

```bash
wpscli catalog drives
wpscli drives list-files --path-param drive_id=<id> --path-param parent_id=0 --query page_size=5
```

### Raw 调用

```bash
wpscli raw GET /v7/companies/current --user-token
```

---

## 项目结构

```text
wps-cli/
  src/
    auth.rs              # 授权、token、签名
    auth_commands.rs     # auth 子命令
    doctor.rs            # 诊断命令
    secure_store.rs      # 加密存储（AES-GCM + keyring）
    helpers/             # doc/files/users/dbsheet 等业务 helper
    skill_runtime.rs     # scope preflight + state/log
  skills/                # SKILL.md（执行契约）
  docs/                  # 使用文档
  descriptors/           # WPS 服务描述符
```

---

## 文档索引

- docs 索引：`docs/README.md`
- 文档框架：`docs/docs-framework.md`
- 拆分命令文档索引：`docs/commands/README.md`
- 全量命令参考（汇总版）：`docs/wpscli-command-reference.md`
- DBSheet 指南：`docs/wpscli-dbsheet-guide.md`
- Skill 契约：
  - `skills/wps-doc-rw/SKILL.md`
  - `skills/wps-app-files/SKILL.md`
  - `skills/wps-users/SKILL.md`

---

## 开发

```bash
cargo check
cargo run --bin wpscli -- --help
cargo run --bin wpscli -- doctor
```

如命令与源码不一致，重新安装：

```bash
cargo install --path . --bin wpscli --force
```
