---
name: wps-doclibs
version: 1.0.0
description: "WPS OpenAPI service: doclibs"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli doclibs --help"
---

# doclibs service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli doclibs <endpoint> [flags]
```

## API Resources

### doclibs

  - `get-doclib-list` — 获取团队文档库列表 (`GET` `/v7/doclibs`; scopes: `kso.doclib.readwrite, kso.doclib.readwrite, kso.doclib.read`)

## Discovering Commands

```bash
wpscli doclibs --help
wpscli schema doclibs
```
