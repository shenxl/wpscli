---
name: wps-meeting_room_privileged_users
version: 1.0.0
description: "WPS OpenAPI service: meeting_room_privileged_users"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli meeting_room_privileged_users --help"
    auth_types: ["app"]
---

# meeting_room_privileged_users service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli meeting_room_privileged_users <endpoint> [flags]
```

## API Resources

### meeting_room_privileged_users

  - `create-meeting-room-privileged-user` — 创建会议室白名单用户 (`GET` `/v7/meeting_room_privileged_users/create`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)
  - `delete-meeting-room-privileged-user` — 删除会议室白名单用户 (`GET` `/v7/meeting_room_privileged_users/delete`; scopes: `kso.meeting_rooms.readwrite`; auth: `app`)
  - `get-meeting-room-privileged-user-list` — 获取会议室白名单用户列表 (`GET` `/v7/meeting_room_privileged_users`; scopes: `kso.meeting_rooms.read, kso.meeting_rooms.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli meeting_room_privileged_users --help
wpscli schema meeting_room_privileged_users
```
