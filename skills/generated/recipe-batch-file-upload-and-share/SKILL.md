---
name: recipe-batch-file-upload-and-share
version: 1.0.0
description: "批量上传文件后，统一授予协作者访问权限。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "productivity"
---

# 批量上传并授权

批量上传文件后，统一授予协作者访问权限。

- Services: `files`, `drives`
- Auth sequence: `user -> app`

## Steps

1. 批量上传文件: `wpscli files transfer --mode upload --batch-file <manifest.json> --auth-type user`
2. 查询角色: `wpscli drives list-roles --auth-type app --path-param drive_id=<drive_id>`
3. 批量授权: `wpscli files batch-create-file-permission --auth-type app --path-param drive_id=<drive_id> --path-param file_id=<file_id> --body '{...}'`
