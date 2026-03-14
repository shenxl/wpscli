---
name: wps-cookie_contacts
version: 1.0.0
description: "WPS OpenAPI service: cookie_contacts"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli cookie_contacts --help"
    auth_types: ["cookie"]
---

# cookie_contacts service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli cookie_contacts <endpoint> [flags]
```

## API Resources

### users

  - `search-users-cookie` — 通过 cookie 会话搜索用户（私有 V7） (`GET` `/v7/users/search`; scopes: `-`; auth: `cookie-only`)

## Discovering Commands

```bash
wpscli cookie_contacts --help
wpscli schema cookie_contacts
```
