---
name: wps-meeting_rooms
version: 1.0.0
description: "WPS OpenAPI service: meeting_rooms"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli meeting_rooms --help"
    auth_types: ["app"]
---

# meeting_rooms service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli meeting_rooms <endpoint> [flags]
```

## API Resources

### meeting_rooms

  - `batch-get-meeting-room` — 批量查询会议室详情 (`GET` `/v7/meeting_rooms/batch_get`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`; auth: `app`)
  - `create-meeting-room` — 创建会议室 (`GET` `/v7/meeting_rooms/create`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)
  - `delete-meeting-room` — 删除会议室 (`GET` `/v7/meeting_rooms/{room_id}/delete`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)
  - `get-meeting-room` — 查询会议室详情 (`GET` `/v7/meeting_rooms/{room_id}`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`; auth: `app`)
  - `get-meeting-room-disable` — 获取会议室禁用信息 (`GET` `/v7/meeting_rooms/{room_id}/disable`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`; auth: `app`)
  - `get-meeting-room-list` — 查询会议室列表 (`GET` `/v7/meeting_rooms`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`; auth: `app`)
  - `post-meeting-room-disable` — 设置会议室禁用 (`GET` `/v7/meeting_rooms/{room_id}/disable`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)
  - `release-meeting-room` — 提前释放会议室 (`GET` `/v7/meeting_rooms/{room_id}/release`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)
  - `search-meeting-room` — 搜索会议室 (`GET` `/v7/meeting_rooms/search`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`; auth: `app`)
  - `update-meeting-room` — 更新会议室 (`GET` `/v7/meeting_rooms/{room_id}/update`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli meeting_rooms --help
wpscli schema meeting_rooms
```
