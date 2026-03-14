---
name: recipe-analyze-meeting-minutes
version: 1.0.0
description: "读取纪要与转写内容，提取待办后写入智能文档。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "knowledge"
---

# 会议纪要分析并回写文档

读取纪要与转写内容，提取待办后写入智能文档。

- Services: `meetings`, `minutes`, `documents`
- Auth sequence: `user -> user -> user`

## Steps

1. 获取会议纪要列表: `wpscli minutes list-minutes --auth-type user --path-param meeting_id=<meeting_id>`
2. 获取纪要全文: `wpscli minutes get-minute-transcript --auth-type user --path-param meeting_id=<meeting_id> --path-param minute_id=<minute_id>`
3. 提取决策与待办并生成 markdown
4. 写入文档: `wpscli airpage write-md --file-id <file_id> --content-file <report.md> --auth-type user`
