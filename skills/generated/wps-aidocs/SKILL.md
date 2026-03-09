---
name: wps-aidocs
version: 1.0.0
description: "WPS OpenAPI service: aidocs"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli aidocs --help"
---

# aidocs service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli aidocs <endpoint> [flags]
```

## API Resources

### aidocs

  - `doclib-list` — 获取AI团队列表 (`GET` `/v7/aidocs/doclib_list`; scopes: `kso.aidocs.readwrite`)
  - `doclib-status` — 获取开启AI团队列表状态（进度） (`GET` `/v7/aidocs/doclib/switch/status`; scopes: `kso.aidocs.readwrite`)
  - `doclib-switch` — 开启AI团队 (`POST` `/v7/aidocs/doclib/switch`; scopes: `kso.aidocs.readwrite`)
  - `extract-commit` — 内容智能抽取-提交任务 (`POST` `/v7/aidocs/extract/commit`; scopes: `kso.aidocs_extract.readwrite, kso.aidocs_extract.readwrite`)
  - `extract-res` — 内容智能抽取-获取结果 (`POST` `/v7/aidocs/extract/res`; scopes: `kso.aidocs_extract.readwrite, kso.aidocs_extract.readwrite`)

## Discovering Commands

```bash
wpscli aidocs --help
wpscli schema aidocs
```
