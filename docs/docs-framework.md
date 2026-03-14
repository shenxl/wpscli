# 文档框架（Docs Framework）

`wpscli` 是一个对文档一致性与质量可观测要求很高的 CLI 项目。  
本项目采用以下成熟方案：

- **Docs as Code**：文档与代码同仓库管理
- **命令帮助自动生成**：从真实 `clap --help` 输出生成文档，避免手写漂移
- **可检查模式（--check）**：用于提交前/CI 校验文档是否过期
- **质量门禁命令**：通过 `wpscli quality` 提供全量 service/endpoint 基线验证

---

## 1) 设计原则

- 命令参数的唯一事实来源是代码（`clap` 定义）
- 命令文档由脚本自动生成，不手工维护参数细节
- 场景化文档（如 dbsheet 专题）可手写，但需链接到命令参考

---

## 2) 目录约定

- 自动生成命令文档：`docs/commands/*.md`
- 命令文档索引：`docs/commands/README.md`
- 框架说明：`docs/docs-framework.md`
- 场景专题文档：`docs/*.md`（如 `wpscli-dbsheet-guide.md`）

---

## 3) 生成命令文档

执行：

```bash
python3 scripts/generate_docs.py
```

会自动生成：

- `docs/commands/01-root.md`
- `docs/commands/02-auth.md`
- `docs/commands/03-doc.md`
- `docs/commands/04-files-users.md`
- `docs/commands/05-dbsheet.md`
- `docs/commands/06-integrations.md`
- `docs/commands/07-system.md`
- `docs/commands/README.md`

其中 `07-system.md` 已覆盖：

- `schema`
- `catalog`
- `raw`
- `generate-skills`
- `completions`
- `ui`
- `doctor`
- `quality`

---

## 4) 检查文档是否过期

执行：

```bash
python3 scripts/generate_docs.py --check
```

- 若文档已是最新：退出码 `0`
- 若有命令变更未同步文档：退出码 `1` 并提示需要更新的文件

---

## 5) 提交前自动校验（可选但推荐）

安装 git hooks：

```bash
bash scripts/install-git-hooks.sh
```

安装后 `pre-commit` 会自动执行：

```bash
python3 scripts/generate_docs.py --check
```

---

## 6) 推荐工作流

1. 修改命令代码（新增参数/子命令/质量门禁）
2. 运行 `python3 scripts/generate_docs.py`
3. 运行 `wpscli quality`，确认静态/构造门禁通过
4. 检查 `docs/commands` 变化
5. 提交代码与文档变更

---

## 7) 为什么这个方案成熟

- 与 `clap` 原生帮助保持一致，稳定、低维护成本
- 与 Git 工作流天然兼容（`--check` 可接 hook/CI）
- 可在不引入重型依赖的情况下快速落地并长期演进
