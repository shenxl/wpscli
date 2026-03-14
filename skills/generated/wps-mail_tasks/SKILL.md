---
name: wps-mail_tasks
version: 1.0.0
description: "WPS OpenAPI service: mail_tasks"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli mail_tasks --help"
    auth_types: ["app"]
---

# mail_tasks service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli mail_tasks <endpoint> [flags]
```

## API Resources

### mail_tasks

  - `get-mail-task` — 获取邮箱任务信息 (`GET` `/v7/mail_tasks/{task_id}`; scopes: `kso.mail_contact.readwrite, kso.mail_contact.read`; auth: `app`)

## Discovering Commands

```bash
wpscli mail_tasks --help
wpscli schema mail_tasks
```
