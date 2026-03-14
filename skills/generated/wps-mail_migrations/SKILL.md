---
name: wps-mail_migrations
version: 1.0.0
description: "WPS OpenAPI service: mail_migrations"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli mail_migrations --help"
    auth_types: ["app"]
---

# mail_migrations service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli mail_migrations <endpoint> [flags]
```

## API Resources

### mail_migrations

  - `add-migration-member` — 在搬家任务中添加成员 (`GET` `/v7/mail_migrations/{migration_id}/tasks`; scopes: `kso.mail_migration.readwrite`; auth: `app`)
  - `get-migration-list` — 获取搬家任务列表 (`GET` `/v7/mail_migrations/{migration_id}/tasks`; scopes: `kso.mail_migration.readwrite, kso.mail_migration.read`; auth: `app`)
  - `start-migration` — Exchange管理员为成员启动搬家 (`GET` `/v7/mail_migrations/{migration_id}/tasks/{task_id}/start`; scopes: `kso.mail_migration.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli mail_migrations --help
wpscli schema mail_migrations
```
