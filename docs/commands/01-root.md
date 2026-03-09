# 顶层命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

`wpscli` 顶层入口与全局参数。

---

## 根命令帮助

```bash
wpscli --help
```

```text
WPS OpenAPI 命令行（面向开发者与 AI Agent）。

本 CLI 提供三层调用能力：
- 业务技能/助手命令（高层语义任务）
- 描述符驱动动态 API 命令（覆盖面完整）
- raw 原始 HTTP 调用（任意路径）

可先使用 `wpscli catalog` 快速发现服务与端点。

Usage: wpscli [OPTIONS] [COMMAND]

Commands:
  auth             管理 WPS 授权（支持引导模式）
  schema           查看服务/端点的参数与结构定义
  catalog          按服务或 show 分类列出可用 API
  raw              直接调用任意 WPS API 路径
  generate-skills  根据描述符生成 SKILL.md 文档
  completions      生成 shell 自动补全脚本
  ui               显示交互引导 ASCII 场景
  doctor           执行本地诊断（安装状态/鉴权就绪/安全检查）
  help             Print this message or the help of the given subcommand(s)

Options:
      --output <output>
          输出格式：json（默认）/compact/table
          
          [default: json]
          [possible values: json, compact, table]

  -h, --help
          Print help (see a summary with '-h')

示例：
  wpscli auth guide
  wpscli auth login --user --mode local
  wpscli auth status
  wpscli catalog drives
  wpscli drives list-files --path-param drive_id=<id> --path-param parent_id=0 --query page_size=5
  wpscli raw GET /v7/drives --query allotee_type=user --query page_size=5
```
