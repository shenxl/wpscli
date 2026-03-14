---
name: wps-attendance
version: 1.0.0
description: "WPS OpenAPI service: attendance"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli attendance --help"
    auth_types: ["app", "user"]
---

# attendance service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli attendance <endpoint> [flags]
```

## API Resources

### attendance

  - `batch-get-punches` — 批量查询打卡记录 (`GET` `/v7/attendance/punches/batch_get`; scopes: `kso.attendance_punch_record.read`; auth: `both`)
  - `delete-approval-record` — 删除假勤审批记录 (`GET` `/v7/attendance/approval_records/delete`; scopes: `kso.attendance_approval_record.readwrite`; auth: `both`)
  - `get-group-detail` — 获取考勤组详情 (`GET` `/v7/attendance/groups/{group_id}`; scopes: `kso.attendance_group.read`; auth: `both`)
  - `import-approval-record` — 导入假勤审批记录 (`GET` `/v7/attendance/approval_records/import`; scopes: `kso.attendance_approval_record.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli attendance --help
wpscli schema attendance
```
