---
name: wps-wiki
version: 1.0.0
description: "WPS OpenAPI service: wiki"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli wiki --help"
    auth_types: ["user"]
---

# wiki service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli wiki <endpoint> [flags]
```

## API Resources

### wiki

  - `batch-copy-space-file` — 批量复制知识库文件（夹） (`GET` `/v7/wiki/spaces/{space_id}/files/batch_copy`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `batch-create-space-member` — 批量添加知识库成员 (`GET` `/v7/wiki/spaces/{space_id}/members/batch_create`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `batch-delete-space-member` — 批量删除知识库成员 (`GET` `/v7/wiki/spaces/{space_id}/members/batch_delete`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `batch-move-space-file` — 批量移动知识库文件（夹） (`GET` `/v7/wiki/spaces/{space_id}/files/batch_move`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `create-space` — 创建知识库 (`GET` `/v7/wiki/spaces/create`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `create-space-file` — 创建知识库节点 (`GET` `/v7/wiki/spaces/{space_id}/files/{parent_id}/create`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `get-space` — 获取知识库 (`GET` `/v7/wiki/spaces/{space_id}/meta`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `get-space-file` — 获取知识库节点信息 (`GET` `/v7/wiki/spaces/{space_id}/files/{file_id}/meta`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `get-space-file-list` — 获取知识库子节点列表 (`GET` `/v7/wiki/spaces/{space_id}/files/{file_id}/children`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `get-space-list` — 获取知识库列表 (`GET` `/v7/wiki/spaces`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `get-space-member-list` — 获取知识库成员列表 (`GET` `/v7/wiki/spaces/{space_id}/members`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `get-task` — 获取知识库异步任务信息 (`GET` `/v7/wiki/tasks/{task_id}`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `move-space-file` — 移动知识库文件（夹） (`GET` `/v7/wiki/spaces/{space_id}/files/move`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `renmae-space-file` — 重命名知识库节点 (`GET` `/v7/wiki/spaces/{space_id}/files/{file_id}/rename`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `search-space` — 知识库文件搜索 (`GET` `/v7/wiki/search`; scopes: `kso.wiki.readwrite`; auth: `user`)
  - `update-space-seting` — 更新知识库设置 (`GET` `/v7/wiki/spaces/{space_id}/settings`; scopes: `kso.wiki.readwrite`; auth: `user`)

## Discovering Commands

```bash
wpscli wiki --help
wpscli schema wiki
```
