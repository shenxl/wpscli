# 应用文件与用户助手命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

`files` 和 `users` 业务助手命令。

---

## files

```bash
wpscli files --help
```

```text
应用目录与文件助手（wps-app-files）

Usage: files [COMMAND]

Commands:
  list-apps   列出 应用/openclaw 下的应用目录
  ensure-app  确保应用目录存在
  create      创建应用文件（会自动创建目录）
  add-file    在已有应用下新增文件 [aliases: create-file]
  list-files  列出某应用下的文件
  get         查询文件信息
  upload      上传本地文件（请求上传->存储上传->提交完成）
  download    下载文件到本地
  state       查看本地状态仓库（registry/log）
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli files list-apps --user-token
  wpscli files create --app "Demo" --file "日报.otl" --user-token
  wpscli files list-files --app "Demo" --user-token
```

## users

```bash
wpscli users --help
```

```text
用户与组织架构助手（wps-users）

Usage: users [COMMAND]

Commands:
  scope    查看通讯录权限范围（org）
  depts    列出指定部门的子部门
  members  列出部门成员
  user     查询指定用户详情
  list     按条件查询用户列表
  find     按姓名关键字搜索用户
  sync     同步组织架构摘要（轻量版）
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli users scope --user-token
  wpscli users depts --dept-id root --user-token
  wpscli users members --dept-id 123 --recursive true --user-token
  wpscli users find --name 张三 --user-token
```
