---
name: wps-partners
version: 1.0.0
description: "WPS OpenAPI service: partners"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli partners --help"
    auth_types: ["app", "user"]
---

# partners service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli partners <endpoint> [flags]
```

## API Resources

### partners

  - `get-partners` — 获取关联组织列表 (`GET` `/v7/partners`; scopes: `kso.contact.readwrite, kso.contact.read`; auth: `both`)
  - `get-partners-dept-list` — 获取关联组织下的部门列表 (`GET` `/v7/partners/{partner_id}/depts/{dept_id}/children`; scopes: `kso.contact.readwrite, kso.contact.read`; auth: `both`)
  - `get-partners-dept-user-list` — 获取关联组织的指定部门的用户列表 (`GET` `/v7/partners/{partner_id}/depts/{dept_id}/members`; scopes: `kso.contact.readwrite, kso.contact.read`; auth: `both`)

## Discovering Commands

```bash
wpscli partners --help
wpscli schema partners
```
