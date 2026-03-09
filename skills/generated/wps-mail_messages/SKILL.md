---
name: wps-mail_messages
version: 1.0.0
description: "WPS OpenAPI service: mail_messages"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli mail_messages --help"
---

# mail_messages service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli mail_messages <endpoint> [flags]
```

## API Resources

### mail_messages

  - `advance-search-mail` — 搜索邮件【高级搜索】 (`GET` `/v7/mail_messages/search`; scopes: `kso.mail.readwrite, kso.mail.read`)

## Discovering Commands

```bash
wpscli mail_messages --help
wpscli schema mail_messages
```
