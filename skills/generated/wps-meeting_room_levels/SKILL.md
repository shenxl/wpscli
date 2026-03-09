---
name: wps-meeting_room_levels
version: 1.0.0
description: "WPS OpenAPI service: meeting_room_levels"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli meeting_room_levels --help"
---

# meeting_room_levels service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli meeting_room_levels <endpoint> [flags]
```

## API Resources

### meeting_room_levels

  - `batch-get-meeting-room-level` — 批量查询会议室层级详情 (`GET` `/v7/meeting_room_levels/batch_get`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`)
  - `create-meeting-room-level` — 创建会议室层级 (`GET` `/v7/meeting_room_levels/create`; scopes: `kso.meeting_rooms.readwrite`)
  - `delete-meeting-room-level` — 删除会议室层级 (`GET` `/v7/meeting_room_levels/{room_level_id}/delete`; scopes: `kso.meeting_rooms.readwrite`)
  - `get-meeting-room-level` — 查询会议室层级详情 (`GET` `/v7/meeting_room_levels/{room_level_id}`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`)
  - `get-meeting-room-level-list` — 查询会议室层级列表 (`GET` `/v7/meeting_room_levels`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`)
  - `update-meeting-room-level` — 更新会议室层级 (`GET` `/v7/meeting_room_levels/{room_level_id}/update`; scopes: `kso.meeting_rooms.readwrite`)

## Discovering Commands

```bash
wpscli meeting_room_levels --help
wpscli schema meeting_room_levels
```
