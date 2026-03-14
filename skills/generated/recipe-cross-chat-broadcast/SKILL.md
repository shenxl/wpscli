---
name: recipe-cross-chat-broadcast
version: 1.0.0
description: "读取源群消息，提取摘要并分发到多个目标群。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "communication"
---

# 跨群摘要广播

读取源群消息，提取摘要并分发到多个目标群。

- Services: `cookie_im`, `messages`
- Auth sequence: `cookie -> user`

## Steps

1. 读取源群历史: `wpscli cookie_im search-messages-cookie --auth-type cookie --query chat_ids=<source_chat_id>`
2. 生成摘要文本
3. 循环发送到目标群: `wpscli messages create-message --auth-type user --path-param chat_id=<target_chat_id> --body '{...}'`
