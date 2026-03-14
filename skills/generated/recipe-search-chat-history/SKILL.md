---
name: recipe-search-chat-history
version: 1.0.0
description: "基于私有 IM 搜索接口执行会话和消息的全局检索。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "communication"
---

# 搜索聊天历史（Cookie 场景）

基于私有 IM 搜索接口执行会话和消息的全局检索。

- Services: `cookie_im`
- Auth sequence: `cookie -> cookie`

## Steps

1. 搜索会话: `wpscli cookie_im search-chats-cookie --auth-type cookie --query query=项目群`
2. 搜索消息: `wpscli cookie_im search-messages-cookie --auth-type cookie --query keyword=需求`
3. 按会话和时间段二次过滤结果
