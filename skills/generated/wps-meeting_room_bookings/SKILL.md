---
name: wps-meeting_room_bookings
version: 1.0.0
description: "WPS OpenAPI service: meeting_room_bookings"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli meeting_room_bookings --help"
---

# meeting_room_bookings service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli meeting_room_bookings <endpoint> [flags]
```

## API Resources

### meeting_room_bookings

  - `batch-get-meeting-room-booking` — 批量查询会议室预约 (`GET` `/v7/meeting_room_bookings/batch_get`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`)
  - `update-meeting-booking-status` — 更新会议室预约状态 (`GET` `/v7/meeting_room_bookings/{booking_id}/update_status`; scopes: `kso.meeting_rooms.readwrite`)

## Discovering Commands

```bash
wpscli meeting_room_bookings --help
wpscli schema meeting_room_bookings
```
