# 系统命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

框架级命令：schema/catalog/raw/generate-skills/completions/ui/doctor/quality。

---

## schema

```bash
wpscli schema --help
```

```text
查看服务/端点的参数与结构定义

Usage: wpscli schema [OPTIONS] <service> [endpoint]

Arguments:
  <service>   
  [endpoint]  

Options:
      --mode <mode>                    输出模式：raw=原始 descriptor，invoke=执行导向 schema（含 command/template） [default: raw] [possible values: raw, invoke]
      --output <output>                输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
      --emit-template <emit-template>  将 invoke_template 写入文件（需同时提供 endpoint）
  -h, --help                           Print help

示例：
  wpscli schema drives
  wpscli schema drives list-files
  wpscli schema drives list-files --mode invoke
  wpscli schema drives list-files --mode invoke --emit-template /tmp/list_files_template.json
```

## catalog

```bash
wpscli catalog --help
```

```text
按服务或 show 分类列出可用 API

Usage: wpscli catalog [OPTIONS] [service]

Arguments:
  [service]  

Options:
      --mode <mode>      分组模式：show=按 show.json 层级，service=按服务平铺，ai=机器可读索引 [default: show] [possible values: show, service, ai]
      --output <output>  输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
  -h, --help             Print help

示例：
  wpscli catalog
  wpscli catalog --mode service
  wpscli catalog --mode ai
  wpscli catalog drives
```

## raw

```bash
wpscli raw --help
```

```text
直接调用任意 WPS API 路径

Usage: wpscli raw [OPTIONS] <method> <path>

Arguments:
  <method>  [possible values: GET, POST, PUT, PATCH, DELETE]
  <path>    

Options:
      --output <output>        输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
  -q, --query <query>          
  -H, --header <header>        
      --body <body>            
      --auth-type <auth-type>  [default: app] [possible values: app, user, cookie]
      --user-token             快捷方式：等价于 --auth-type user
      --dry-run                
      --retry <retry>          [default: 1]
  -h, --help                   Print help

示例：
  wpscli raw GET /v7/drives --query allotee_type=user --query page_size=5
  wpscli raw POST /v7/messages/create --user-token --body '{"chat_id":"xxx","text":"hello"}'
```

## generate-skills

```bash
wpscli generate-skills --help
```

```text
根据描述符生成 SKILL.md 文档

Usage: wpscli generate-skills [OPTIONS]

Options:
      --out-dir <out-dir>  [default: skills/generated]
      --output <output>    输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
  -h, --help               Print help

示例：
  wpscli generate-skills --out-dir skills/generated
```

## completions

```bash
wpscli completions --help
```

```text
生成 shell 自动补全脚本

Usage: wpscli completions [OPTIONS] <shell>

Arguments:
  <shell>  [possible values: bash, zsh, fish, powershell, elvish]

Options:
      --output <output>  输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
  -h, --help             Print help

示例：
  wpscli completions zsh > ~/.zfunc/_wpscli
```

## ui

```bash
wpscli ui --help
```

```text
显示交互引导 ASCII 场景

Usage: wpscli ui [OPTIONS] [scene]

Arguments:
  [scene]  场景名称 [default: all] [possible values: intro, features, framework, setup, config, format, outro, all]

Options:
      --output <output>  输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
  -h, --help             Print help

示例：
  wpscli ui all
  wpscli ui framework
```

## doctor

```bash
wpscli doctor --help
```

```text
执行本地诊断（安装状态/鉴权就绪/安全检查）

Usage: wpscli doctor [OPTIONS]

Options:
      --output <output>  输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
  -h, --help             Print help

示例：
  wpscli doctor
```

## quality

```bash
wpscli quality --help
```

```text
运行描述符质量门禁（静态/构造/连通抽样）

Usage: wpscli quality [OPTIONS]

Options:
      --connectivity-sample <connectivity-sample>
          连通抽样数量（仅 GET + 无必填参数端点，0=跳过） [default: 0]
      --output <output>
          输出格式：json（默认）/compact/table [default: json] [possible values: json, compact, table]
      --connectivity-auth <connectivity-auth>
          连通抽样鉴权策略 [default: auto] [possible values: auto, app, user, cookie]
  -h, --help
          Print help

示例：
  wpscli quality
  wpscli quality --connectivity-sample 10
  wpscli quality --connectivity-sample 10 --connectivity-auth user
```
