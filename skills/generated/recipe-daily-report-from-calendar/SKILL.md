---
name: recipe-daily-report-from-calendar
version: 1.0.0
description: "聚合日历和聊天信息，生成日报并发送。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "productivity"
---

# 日程与消息汇总日报

聚合日历和聊天信息，生成日报并发送。

- Services: `calendars`, `cookie_im`, `chats`, `messages`
- Auth sequence: `user -> cookie -> user`

## Steps

1. 查询今日日程: `wpscli calendars list-events --auth-type user --path-param calendar_id=primary --query ...`
2. 检索今日关键消息: `wpscli cookie_im search-messages-cookie --auth-type cookie --query keyword=日报`
3. 整理为 markdown 报告
4. 发送到目标会话: `wpscli messages create-message --auth-type user --path-param chat_id=<chat_id> --body '{...}'`
