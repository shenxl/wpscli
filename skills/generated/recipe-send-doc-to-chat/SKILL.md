---
name: recipe-send-doc-to-chat
version: 1.0.0
description: "先上传文件再通过 cookie 私有接口发送云文档消息。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "communication"
---

# 上传文件并发送到群聊

先上传文件再通过 cookie 私有接口发送云文档消息。

- Services: `files`, `links`, `cookie_im`
- Auth sequence: `user -> user -> cookie`

## Steps

1. 上传文件: `wpscli files upload --app <app> --file <local_path> --auth-type user`
2. 获取分享元信息: `wpscli links get-link-meta --auth-type user --path-param link_id=<link_id>`
3. 发送云文档消息: `wpscli cookie_im send-cloud-file-cookie --auth-type cookie --path-param chat_id=<chat_id> --body '{...}'`
