---
name: wps-free_busy_list
version: 1.0.0
description: "WPS OpenAPI service: free_busy_list"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli free_busy_list --help"
---

# free_busy_list service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli free_busy_list <endpoint> [flags]
```

## API Resources

### free_busy_list

  - `get-main-calendar-freebusy` — 查询主日历日程忙闲 (`GET` `/v7/free_busy_list`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)

## Discovering Commands

```bash
wpscli free_busy_list --help
wpscli schema free_busy_list
```
