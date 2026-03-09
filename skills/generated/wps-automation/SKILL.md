---
name: wps-automation
version: 1.0.0
description: "WPS OpenAPI service: automation"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli automation --help"
---

# automation service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli automation <endpoint> [flags]
```

## API Resources

### automation

  - `list-automation` — 列出自动化流程 (`GET` `/v7/automation/{file_id}/workflows`; scopes: `kso.automation.readwrite, kso.automation.read`)
  - `update-automation-status` — 更新自动化流程状态 (`GET` `/v7/automation/{file_id}/workflows/{workflow_id}/status`; scopes: `kso.automation.readwrite`)

## Discovering Commands

```bash
wpscli automation --help
wpscli schema automation
```
