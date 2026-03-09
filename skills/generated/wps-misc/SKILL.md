---
name: wps-misc
version: 1.0.0
description: "WPS OpenAPI service: misc"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli misc --help"
---

# misc service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli misc <endpoint> [flags]
```

## API Resources

### api

  - `signature-description-wps-3` — 签名说明（WPS-3） (`GET` `/api/v1/dosomething?name=xiaoming&age=18`; scopes: `-`)

## Discovering Commands

```bash
wpscli misc --help
wpscli schema misc
```
