---
name: wps-workflow
version: 1.0.0
description: "WPS OpenAPI service: workflow"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli workflow --help"
    auth_types: ["app", "user"]
---

# workflow service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli workflow <endpoint> [flags]
```

## API Resources

### workflow

  - `add-sign-approval-task` — 审批任务加签 (`GET` `/v7/workflow/approval_tasks/{id}/add_sign`; scopes: `kso.workflow_approval_task.readwrite`; auth: `both`)
  - `approve-approval-task` — 同意审批任务 (`GET` `/v7/workflow/approval_tasks/{id}/approve`; scopes: `kso.workflow_approval_task.readwrite, kso.workflow_approval_task.readwrite`; auth: `both`)
  - `comment-approval-instance` — 评论 (`GET` `/v7/workflow/approval_instances/{id}/comment`; scopes: `kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `create-approval-definition` — 创建审批定义 (`GET` `/v7/workflow/approval_defines`; scopes: `kso.workflow_approval_define.readwrite`; auth: `both`)
  - `create-approval-instance` — 发起审批 (`GET` `/v7/workflow/approval_instances`; scopes: `kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `create-external-approval-define` — 创建三方审批定义 (`GET` `/v7/workflow/external_defines`; scopes: `kso.workflow_approval_define.readwrite`; auth: `both`)
  - `create-external-approval-instance` — 同步三方审批实例 (`GET` `/v7/workflow/external_instances`; scopes: `kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `delete-approval-definition` — 删除审批定义 (`GET` `/v7/workflow/approval_defines/{id}/delete`; scopes: `kso.workflow_approval_define.readwrite`; auth: `both`)
  - `get-approval-definition` — 查询审批定义 (`GET` `/v7/workflow/approval_defines/{id}`; scopes: `kso.workflow_approval_define.readwrite, kso.workflow_approval_define.read, kso.workflow_approval_define.readwrite, kso.workflow_approval_define.read`; auth: `both`)
  - `get-approval-definition-list` — 分页查询审批定义列表 (`GET` `/v7/workflow/approval_defines`; scopes: `kso.workflow_approval_define.readwrite, kso.workflow_approval_define.read`; auth: `both`)
  - `get-approval-definition-list-v2` — 查询审批定义列表 (`GET` `/v7/workflow/approval_defines/batch_get`; scopes: `kso.workflow_approval_define.read, kso.workflow_approval_define.readwrite`; auth: `both`)
  - `get-approval-instance` — 查询审批实例 (`GET` `/v7/workflow/approval_instances/{id}`; scopes: `kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite, kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `get-approval-instance-brief-list` — 批量查询审批实例简要信息 (`GET` `/v7/workflow/approval_instances/batch_get`; scopes: `kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite, kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `get-approval-instance-cc-list` — 查询审批实例抄送列表 (`GET` `/v7/workflow/approval_instances/batch_get_cc`; scopes: `kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `get-approval-instance-list` — 批量查询审批实例详情 (`GET` `/v7/workflow/approval_instances`; scopes: `kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite, kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `get-approval-instance-timeline-list` — 查询审批实例操作记录列表 (`GET` `/v7/workflow/approval_instances/batch_get_timeline`; scopes: `kso.workflow_approval_instance.read, kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `get-approval-task-list` — 查询审批任务列表 (`GET` `/v7/workflow/approval_tasks/batch_get`; scopes: `kso.workflow_approval_task.readwrite`; auth: `both`)
  - `get-external-approval-define` — 获取三方审批定义 (`GET` `/v7/workflow/external_defines/{define_code}`; scopes: `kso.workflow_approval_define.read, kso.workflow_approval_define.readwrite`; auth: `both`)
  - `get-external-approval-task-list` — 查询三方审批任务状态 (`GET` `/v7/workflow/external_tasks`; scopes: `kso.workflow_approval_task.readwrite`; auth: `both`)
  - `get-user-manage-approval-definition-list` — 分页查询用户管理审批定义列表 (`GET` `/v7/workflow/approval_defines/user_manage`; scopes: `kso.workflow_approval_define.readwrite, kso.workflow_approval_define.read`; auth: `both`)
  - `reject-approval-task` — 拒绝审批任务 (`GET` `/v7/workflow/approval_tasks/{id}/reject`; scopes: `kso.workflow_approval_task.readwrite, kso.workflow_approval_task.readwrite`; auth: `both`)
  - `revoke-approval-instance` — 撤销审批实例 (`GET` `/v7/workflow/approval_instances/{id}/revoke`; scopes: `kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `rollback-approval-task` — 退回审批任务 (`GET` `/v7/workflow/approval_tasks/{id}/rollback`; scopes: `kso.workflow_approval_task.readwrite, kso.workflow_approval_task.readwrite`; auth: `both`)
  - `send-approval-message` — 发送审批机器人消息 (`GET` `/v7/workflow/message/send`; scopes: `kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `simulate-approval-instance` — 模拟审批流程 (`GET` `/v7/workflow/approval_instances/simulation`; scopes: `kso.workflow_approval_instance.readwrite`; auth: `both`)
  - `transfer-approval-task` — 转交审批任务 (`GET` `/v7/workflow/approval_tasks/{id}/transfer`; scopes: `kso.workflow_approval_task.readwrite, kso.workflow_approval_task.readwrite`; auth: `both`)
  - `update-approval-definition` — 更新审批定义 (`GET` `/v7/workflow/approval_defines/{id}/update`; scopes: `kso.workflow_approval_define.readwrite`; auth: `both`)
  - `update-approval-definition-setting` — 更新审批定义基础设置 (`GET` `/v7/workflow/approval_defines/{id}/settings`; scopes: `kso.workflow_approval_define.readwrite`; auth: `both`)
  - `update-approval-message` — 更新审批机器人消息 (`GET` `/v7/workflow/message/update`; scopes: `kso.workflow_approval_instance.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli workflow --help
wpscli schema workflow
```
