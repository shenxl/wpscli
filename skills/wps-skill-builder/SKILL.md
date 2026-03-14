---
name: wps-skill-builder
description: Create and iterate business-oriented WPS skills by discovering APIs across services, composing helper workflows, and generating Claude-compatible skill packages with eval scaffolding.
dependencies: []
---

# WPS Skill Builder (Skill Creator)

这个技能不是“接口翻译器”，而是“业务技能创建器”：

- 先发现全量 API 能力（跨 service）
- 再按业务目标拼装 helper 工作流（类似 `src/helpers/` 的组合方式）
- 最后生成 Claude 可复用技能包 + 评测脚手架

## 适用场景

- 你要从 0 创建一个“业务技能”而非单接口调用说明。
- 你要把 `doc/files/users/dbsheet/...` 这些能力组合成完整流程。
- 你要迭代优化技能（补充测试 prompt、失败恢复策略、输出契约）。

## 核心命令

### 1) 全局能力发现（跨 descriptors）

```bash
python3 build_skill.py discover \
  --query "组织通讯录同步并生成分析文档" \
  --helper users \
  --helper doc \
  --limit 30
```

### 2) 创建业务技能（推荐）

```bash
python3 build_skill.py create-business \
  --name wps-org-report-creator \
  --description "Sync org users and generate reporting artifacts with robust recovery paths." \
  --goal "同步组织成员并输出日报文档" \
  --helper users \
  --helper files \
  --helper doc \
  --limit 60 \
  --output-dir ./dist
```

### 3) 兼容模式：接口翻译技能（旧模式）

```bash
python3 build_skill.py create-api \
  --name wps-drives-readonly \
  --description "Read WPS drive metadata and files for analysis workflows." \
  --service drives \
  --include list \
  --limit 12 \
  --output-dir ./dist
```

## 生成产物（业务模式）

- `Skill.md`：Claude frontmatter + 业务触发与执行规则
- `references/HELPER_COMPOSITION.md`：helper 组合蓝图
- `references/API_BASELINE.md`：接口能力基线（method/path/scopes/命令模板）
- `evals/evals.json`：初始测试 prompt
- `commands.json`：机器可读命令信息
- `manifest.json`：生成元数据
- `workspace/README.md`：迭代评测工作区约定

## 迭代方式（简版）

1. 先 `discover` 明确可用接口与 helper 组合
2. 用 `create-business` 生成首版技能包
3. 修改 `evals/evals.json` 增补真实测试场景
4. 按测试结果反复优化 `Skill.md` 的触发条件、失败恢复与输出契约

## 设计原则

- 业务优先：helper 编排优先于低层 raw API
- 结构化输出：方便 agent 自动恢复与串联
- 可验证：默认带 eval 脚手架，便于持续迭代
- 可维护：能力基线来自当前 descriptors / `wpscli`，可重复生成

## 执行约束（减少 Agent 试错）

当技能落地到 Claude Code 时，按下面顺序执行，避免“边猜边调参数”：

1. **优先 workflow/recipe**：先尝试工作流模板，再退到 helper 命令。
2. **禁止直接跳 raw**：只有 helper/service 都缺失能力时才允许 `raw`。
3. **失败分支显式化**：每一步都要有 fallback，不允许无限重试。
4. **写操作先演练**：可行时先 `--dry-run`。

## /tmp 临时文件契约（Claude Code）

Claude Code 在执行过程中常会生成 `/tmp` 参数文件（json/py）。这个行为要变成“标准流程”，而不是偶发：

- 执行前写计划：`/tmp/<skill_name>_plan.json`
- 每步参数：`/tmp/<skill_name>_step_<N>.json`
- 每步结果：`/tmp/<skill_name>_result_<N>.json`

这样可以实现：

- 参数来源可追溯
- 重试行为可复现
- 错误定位可审计

`create-business` 生成的技能会自动带上 `references/WORKFLOW_GUARDRAILS.md`，用于约束上述流程。
