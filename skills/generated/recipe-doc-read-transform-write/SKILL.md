---
name: recipe-doc-read-transform-write
version: 1.0.0
description: "读取原文档，执行结构化抽取后写回新文档。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "productivity"
---

# 文档读取-转换-回写

读取原文档，执行结构化抽取后写回新文档。

- Services: `documents`, `files`
- Auth sequence: `user -> user`

## Steps

1. 读取文档: `wpscli doc read-doc --url <share_url> --format markdown --auth-type user`
2. 执行抽取与转换（本地/agent 处理）
3. 写回目标文档: `wpscli doc write-doc --file-id <target_file_id> --target-format otl --content-file <output.md> --auth-type user`
