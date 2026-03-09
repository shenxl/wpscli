---
name: wps-todo
version: 1.0.0
description: "WPS OpenAPI service: todo"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli todo --help"
---

# todo service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli todo <endpoint> [flags]
```

## API Resources

### todo

  - `batch-create-task` — 批量创建待办任务 (`GET` `/v7/todo/tasks/batch_create`; scopes: `kso.task.readwrite`)
  - `batch-delete-personal-todo` — 删除个人待办 (`GET` `/v7/todo/personal_tasks/batch_delete`; scopes: `kso.task.readwrite`)
  - `batch-update-task` — 批量更新待办任务 (`GET` `/v7/todo/tasks/batch_update`; scopes: `kso.task.readwrite`)
  - `create-category` — 创建待办分类 (`POST` `/v7/todo/categories/create`; scopes: `kso.task.readwrite`)
  - `create-personal-todo` — 创建个人待办 (`GET` `/v7/todo/personal_tasks`; scopes: `kso.task.readwrite`)
  - `create-task` — 创建待办任务 (`POST` `/v7/todo/tasks`; scopes: `kso.task.readwrite`)
  - `delete-category` — 删除待办分类 (`POST` `/v7/todo/categories/{category_id}/delete`; scopes: `kso.task.readwrite`)
  - `delete-task` — 删除待办任务 (`POST` `/v7/todo/tasks/batch_delete`; scopes: `kso.task.readwrite`)
  - `get-category` — 查询待办分类 (`GET` `/v7/todo/categories`; scopes: `kso.task.readwrite, kso.task.read`)
  - `get-personal-todo` — 获取个人待办详情 (`GET` `/v7/todo/personal_tasks/{task_id}`; scopes: `kso.task.readwrite, kso.task.read`)
  - `get-personal-todo-list` — 获取个人待办列表 (`GET` `/v7/todo/personal_tasks/batch_get`; scopes: `kso.task.readwrite, kso.task.read`)
  - `update-category` — 更新待办分类 (`POST` `/v7/todo/categories/{category_id}/update`; scopes: `kso.task.readwrite`)
  - `update-personal-todo` — 更新个人待办 (`GET` `/v7/todo/personal_tasks/{task_id}/update`; scopes: `kso.task.readwrite`)
  - `update-personal-todo-status` — 更新个人待办状态 (`GET` `/v7/todo/personal_tasks/{task_id}/update_status`; scopes: `kso.task.readwrite`)
  - `update-task` — 更新待办任务 (`POST` `/v7/todo/tasks/{task_id}/update`; scopes: `kso.task.readwrite`)

## Discovering Commands

```bash
wpscli todo --help
wpscli schema todo
```
