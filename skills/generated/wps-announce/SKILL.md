---
name: wps-announce
version: 1.0.0
description: "WPS OpenAPI service: announce"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli announce --help"
---

# announce service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli announce <endpoint> [flags]
```

## API Resources

### announce

  - `get-announce-cover-url` — 获取封面图下载链接 (`GET` `/v7/announce/announces/{announce_id}/download`; scopes: `kso.announce.read, kso.announce.read`)
  - `get-announce-detail` — 查询公告详情 (`GET` `/v7/announce/announces/{announce_id}`; scopes: `kso.announce.read, kso.announce.read`)
  - `get-announce-list` — 分页查询公告列表 (`GET` `/v7/announce/announces`; scopes: `kso.announce.read, kso.announce.read`)
  - `get-announce-visibilities` — 分页查询公告可见范围 (`GET` `/v7/announce/announces/{announce_id}/visibilities`; scopes: `kso.announce.read`)

## Discovering Commands

```bash
wpscli announce --help
wpscli schema announce
```
