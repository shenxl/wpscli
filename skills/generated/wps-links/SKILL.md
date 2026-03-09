---
name: wps-links
version: 1.0.0
description: "WPS OpenAPI service: links"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli links --help"
---

# links service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli links <endpoint> [flags]
```

## API Resources

### links

  - `get-link-info` — 获取分享链接信息 (`GET` `/v7/links/{link_id}/meta`; scopes: `kso.file_link.readwrite, kso.file_link.readwrite`)
  - `get-recv-link-list` — 获取接收的分享链接列表 (`GET` `/v7/links/recv`; scopes: `kso.file_link.readwrite, kso.file_link.readwrite`)
  - `get-send-link-list` — 获取发送的分享链接列表 (`GET` `/v7/links/send`; scopes: `kso.file_link.readwrite, kso.file_link.readwrite`)
  - `update-link` — 修改分享链接属性 (`GET` `/v7/links/{link_id}/update`; scopes: `kso.file_link.readwrite, kso.file_link.readwrite`)

## Discovering Commands

```bash
wpscli links --help
wpscli schema links
```
