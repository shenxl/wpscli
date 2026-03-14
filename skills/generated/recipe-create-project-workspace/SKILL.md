---
name: recipe-create-project-workspace
version: 1.0.0
description: "创建目录、任务表和项目说明文档。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "project"
---

# 一键创建项目工作区

创建目录、任务表和项目说明文档。

- Services: `files`, `drives`, `coop_dbsheet`, `documents`
- Auth sequence: `user -> user -> user`

## Steps

1. 创建应用目录: `wpscli files ensure-app --app 项目A --auth-type user`
2. 创建任务表: `wpscli files create --app 项目A --file 任务跟踪.dbt --auth-type user`
3. 初始化字段: `wpscli dbsheet init --file-id <dbt_file_id> --schema-file <schema.yaml> --auth-type user`
4. 创建说明文档: `wpscli files create --app 项目A --file 项目说明.otl --auth-type user`
