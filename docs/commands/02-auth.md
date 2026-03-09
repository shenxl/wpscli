# 认证命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

`auth` 相关子命令，覆盖 setup/login/refresh/status/harden/logout。

---

## auth

```bash
wpscli auth --help
```

```text
管理 WPS 授权（支持引导模式）

Usage: auth [COMMAND]

Commands:
  guide         中文引导式授权（推荐新手）
  setup         保存 AK/SK 与 OAuth 相关配置
  login         获取并保存用户 token，或手动写入 token
  refresh-user  使用 refresh_token 立即刷新 user token
  harden        巡检并加固本地敏感信息存储
  status        查看授权状态、token 有效期与自动刷新就绪度
  logout        清空本地保存的凭据与 token 缓存
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli auth guide
  wpscli auth setup --ak <AK> --sk <SK>
  wpscli auth login --user --mode local
  wpscli auth status
```

## auth setup

```bash
wpscli auth setup --help
```

```text
保存 AK/SK 与 OAuth 相关配置

Usage: auth setup [OPTIONS]

Options:
      --ak <ak>                                        应用 AK（client_id）
      --sk <sk>                                        应用 SK（client_secret）
      --redirect-uri <redirect-uri>                    OAuth 回调地址
      --scope <scope>                                  OAuth scope，多个 scope 用空格分隔
      --oauth-server <oauth-server>                    可选：外部 OAuth 代理服务地址
      --oauth-device-endpoint <oauth-device-endpoint>  可选：设备码授权端点
      --oauth-token-endpoint <oauth-token-endpoint>    可选：Token 交换端点
  -h, --help                                           Print help

示例：
  wpscli auth setup --ak <AK> --sk <SK>
  wpscli auth setup --redirect-uri http://localhost:53682/callback --scope kso.user_base.read
```

## auth login

```bash
wpscli auth login --help
```

```text
获取并保存用户 token，或手动写入 token

Usage: auth login [OPTIONS]

Options:
      --user
          启用 OAuth 用户登录流程
      --ak <ak>
          覆盖使用的 AK
      --sk <sk>
          覆盖使用的 SK
      --oauth-server <oauth-server>
          可选：OAuth 代理服务地址
      --code <code>
          授权码（适用于手动流程）
      --state <state>
          [default: wpscli]
      --mode <mode>
          登录模式：local=本地回调，remote=粘贴回调URL，remote-device=设备码轮询 [default: local] [possible values: local, remote, remote-device]
      --scope <scope>
          本次登录使用的 scope
      --redirect-uri <redirect-uri>
          本次登录使用的回调地址
      --callback-url <callback-url>
          远程模式下粘贴完整回调 URL
      --print-url-only
          仅打印授权链接，不继续交换 token
      --no-open
          不自动打开浏览器
      --no-local-server
          关闭本地回调监听，改用 --code 手动输入
      --timeout-seconds <timeout-seconds>
          等待本地回调超时（秒） [default: 180]
      --poll-interval-seconds <poll-interval-seconds>
          设备码模式轮询间隔（秒） [default: 5]
      --user-token <user-token>
          手动写入 user access_token
      --refresh-token <refresh-token>
          手动写入 refresh_token
  -h, --help
          Print help

示例：
  wpscli auth login --user --mode local
  wpscli auth login --user --mode remote
  wpscli auth login --user --mode remote-device
  wpscli auth login --user-token <ACCESS_TOKEN> --refresh-token <REFRESH_TOKEN>
```

## auth harden

```bash
wpscli auth harden --help
```

```text
巡检并加固本地敏感信息存储

Usage: auth harden [OPTIONS]

Options:
      --apply  执行推荐加固动作（删除旧明文、收紧文件权限）
  -h, --help   Print help

示例：
  wpscli auth harden
  wpscli auth harden --apply
```

## auth status

```bash
wpscli auth status --help
```

```text
查看授权状态、token 有效期与自动刷新就绪度

Usage: auth status

Options:
  -h, --help  Print help

示例：
  wpscli auth status
```
