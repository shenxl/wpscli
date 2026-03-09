---
name: wps-contacts
version: 1.0.0
description: "WPS OpenAPI service: contacts"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli contacts --help"
---

# contacts service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli contacts <endpoint> [flags]
```

## API Resources

### contacts

  - `get-range` — 获取通讯录权限范围 (`GET` `/v7/contacts/permissions_scope`; scopes: `kso.contact.readwrite, kso.contact.read`)

## Discovering Commands

```bash
wpscli contacts --help
wpscli schema contacts
```
