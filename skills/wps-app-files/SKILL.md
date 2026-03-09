---
name: wps-app-files
version: 1.0.0
description: 在 WPS 云文档中创建并管理应用目录与应用文件。该技能是“业务编排型命令”，不仅调用 API，还内置目录补齐、scope 预检、本地状态落盘和操作日志。
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli files --help"
---

# wps-app-files（执行契约）

> PREREQUISITE: 先执行 `wpscli auth guide` 或 `wpscli auth login --user` 完成用户授权。

## 能力边界

- 自动补齐目录：`云文档/应用/openclaw/<app>`
- `create/add-file` 默认冲突策略：`fail`（可选 `rename|replace`）
- 支持嵌套应用路径：`--app "项目A/子模块B"`
- 内置 scope 前置检查（优先 `kso.file.read,kso.file.readwrite`）
- 本地状态仓库存储：
  - `~/.config/wps/skills/wps-app-files/app_registry.json`
  - `~/.config/wps/skills/wps-app-files/file_registry.json`
  - `~/.config/wps/skills/wps-app-files/operation_log.jsonl`

## 命令面

- `wpscli files list-apps --user-token`
- `wpscli files ensure-app --app "我的应用" --user-token`
- `wpscli files create --app "我的应用" --file "数据表.dbt" --user-token`
- `wpscli files add-file --app "我的应用" --file "说明.otl" --user-token`
- `wpscli files list-files --app "我的应用" --user-token`
- `wpscli files get --app "我的应用" --file "数据表.dbt" --user-token`
- `wpscli files state --limit 20`

## 输入/输出 Schema（稳定字段）

- `ensure-app` 输入：
  - 必填：`app`
  - 可选：`drive-id`, `--user-token`, `--dry-run`
- `ensure-app` 输出关键字段：
  - `data.data.drive_id`
  - `data.data.app`
  - `data.data.folder`
  - `data.data.scope_preflight`
  - `data.data.state_paths`

- `create/add-file` 输入：
  - 必填：`app`, `file`
  - 可选：`on-name-conflict`, `drive-id`, `--user-token`, `--dry-run`
- `create/add-file` 输出关键字段：
  - `data.data.created`
  - `data.data.workflow`（编排步骤）
  - `data.data.scope_preflight`
  - `data.data.state_paths`

- `state` 输出关键字段：
  - `data.data.paths`
  - `data.data.registry.apps`
  - `data.data.registry.files`
  - `data.data.recent_operations`

## 幂等性约定

- `ensure-app`：幂等。重复执行返回同一路径目录（不存在则创建）。
- `create/add-file`：默认非幂等（同名冲突会失败）；通过 `--on-name-conflict replace` 可实现“名称维度覆盖幂等”。
- `list/get/state`：只读，不修改远程资源。

## 失败语义与恢复路径

- 统一错误类别（JSON）：
  - `auth`：鉴权问题，先 `wpscli auth status`，再 `wpscli auth login --user`
  - `parameter`：参数问题，检查命令参数和 JSON
  - `permission`：目标资源权限不足，确认文件归属和协作者权限
  - `retryable`：临时网络/限流，可增加 `--retry` 后重试
- scope 不足时会给出 re-auth 引导（`suggested_action` + `scope_preflight.reauth_hint`）

## 示例策略（Agent 推荐）

```bash
# 1) 先 dry-run 预演请求
wpscli files create --app "Demo/分析" --file "结果.otl" --user-token --dry-run

# 2) 实际执行
wpscli files create --app "Demo/分析" --file "结果.otl" --user-token

# 3) 拉取状态仓库（用于追踪与回归）
wpscli files state --limit 50
```

## Golden Cases

见同目录 `GOLDEN_CASES.md`，可配合脚本：

```bash
bash scripts/golden_cases_wps_app_files.sh
```
