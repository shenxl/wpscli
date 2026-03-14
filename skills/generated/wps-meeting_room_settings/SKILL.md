---
name: wps-meeting_room_settings
version: 1.0.0
description: "WPS OpenAPI service: meeting_room_settings"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli meeting_room_settings --help"
    auth_types: ["app"]
---

# meeting_room_settings service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli meeting_room_settings <endpoint> [flags]
```

## API Resources

### meeting_room_settings

  - `batch-get-meeting-room-setting` — 批量查询会议室设置 (`GET` `/v7/meeting_room_settings/batch_get`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`; auth: `app`)
  - `update-meeting-room-setting` — 更新会议室设置 (`GET` `/v7/meeting_room_settings/{room_id}/update`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli meeting_room_settings --help
wpscli schema meeting_room_settings
```
