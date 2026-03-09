---
name: wps-docqa
version: 1.0.0
description: "WPS OpenAPI service: docqa"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli docqa --help"
---

# docqa service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli docqa <endpoint> [flags]
```

## API Resources

### docqa

  - `insight-recall-rank` — 团队文档片段召回 (`GET` `/v7/docqa/instore/recall/rank`; scopes: `kso.docqa.readwrite, kso.aidocs.readwrite, kso.docqa.readwrite, kso.aidocs.readwrite`)

## Discovering Commands

```bash
wpscli docqa --help
wpscli schema docqa
```
