# wps-app-files Golden Cases

以下用例来自 `wpsskill/wps-app-files` 的高频场景，作为 `wpscli files` 的回归验收集。

## Case 1: 应用目录自动补齐

- 命令：`wpscli files ensure-app --app "Demo/子模块" --user-token`
- 期望：
  - 自动创建/复用 `应用/openclaw/Demo/子模块`
  - 返回 `folder.id`
  - 本地 `app_registry.json` 记录应用路径和 drive_id

## Case 2: 创建应用文件（目录不存在时自动补齐）

- 命令：`wpscli files create --app "Demo/子模块" --file "数据表.dbt" --user-token`
- 期望：
  - 完整编排执行：resolve drive -> ensure folder -> create file -> persist registry
  - `workflow` 字段包含编排步骤
  - `file_registry.json` 新增文件记录

## Case 3: 同名冲突策略

- 命令：
  - `wpscli files create --app "Demo" --file "报告.otl" --on-name-conflict fail --user-token`
  - `wpscli files create --app "Demo" --file "报告.otl" --on-name-conflict rename --user-token`
- 期望：
  - `fail` 返回冲突错误
  - `rename` 成功并产生不同文件 ID

## Case 4: 查询能力

- 命令：
  - `wpscli files list-apps --user-token`
  - `wpscli files list-files --app "Demo" --user-token`
  - `wpscli files get --app "Demo" --file "数据表.dbt" --user-token`
- 期望：
  - 返回结构化 JSON
  - 不改变远程状态

## Case 5: 本地状态仓库可追踪

- 命令：`wpscli files state --limit 20`
- 期望：
  - 返回 `paths`、`registry` 和 `recent_operations`
  - 所有执行轨迹可回放和定位

## Case 6: dry-run 可离线回归

- 命令：`wpscli files create --app "Demo" --file "test.otl" --dry-run --user-token`
- 期望：
  - `scope_preflight.check_mode = "skipped_dry_run"`
  - 不发起真实 API 写入

