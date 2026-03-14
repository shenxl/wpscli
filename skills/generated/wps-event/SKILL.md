---
name: wps-event
version: 1.0.0
description: "WPS OpenAPI service: event"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli event --help"
    auth_types: ["app"]
---

# event service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli event <endpoint> [flags]
```

## API Resources

### event

  - `get_event_egress_ip` — 获取事件出口地址 (`GET` `/v7/event/egress_ip`; scopes: `kso.event_egress_ip.read`; auth: `app`)

## Discovering Commands

```bash
wpscli event --help
wpscli schema event
```
