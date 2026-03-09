---
name: wps-calendars
version: 1.0.0
description: "WPS OpenAPI service: calendars"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli calendars --help"
---

# calendars service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli calendars <endpoint> [flags]
```

## API Resources

### calendars

  - `batch-create-calendar-event` — 批量创建基础日程 (`GET` `/v7/calendars/{calendar_id}/events/batch_create`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)
  - `batch-create-calendar-permission` — 批量创建日历权限 (`GET` `/v7/calendars/{calendar_id}/permissions/batch_create`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `batch-create-event-attendee` — 添加日程参与者 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/attendees/batch_create`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)
  - `batch-create-event-meeting-room` — 添加日程会议室 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/meeting_rooms/batch_create`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)
  - `batch-delete-event-attendee` — 删除日程参与者 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/attendees/batch_delete`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)
  - `batch-delete-event-meeting-room` — 删除日程会议室 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/meeting_rooms/batch_delete`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)
  - `batch-get-main-calendar` — 批量查询主日历信息 (`GET` `/v7/calendars/primary/batch_get`; scopes: `kso.calendar.read, kso.calendar.readwrite, kso.calendar.read, kso.calendar.readwrite`)
  - `create-calendar` — 创建日历 (`GET` `/v7/calendars/create`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `create-calendar-event` — 创建日程 (`GET` `/v7/calendars/{calendar_id}/events/create`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)
  - `create-calendar-event-chat` — 创建日程群聊 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/create_chat`; scopes: `kso.calendar_events.readwrite`)
  - `create-calendar-permission` — 创建日历权限 (`GET` `/v7/calendars/{calendar_id}/permissions/create`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `create-timeoff-event` — 创建请假日程 (`GET` `/v7/calendars/primary/timeoff_events/create`; scopes: `kso.calendar_events.readwrite`)
  - `delete-calendar` — 删除日历 (`GET` `/v7/calendars/{calendar_id}/delete`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `delete-calendar-event` — 删除日程 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/delete`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)
  - `delete-calendar-permission` — 删除日历权限 (`GET` `/v7/calendars/{calendar_id}/permissions/{calendar_permission_id}/delete`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `delete-timeoff-event` — 删除请假日程 (`GET` `/v7/calendars/primary/timeoff_events/{timeoff_event_id}/delete`; scopes: `kso.calendar_events.readwrite`)
  - `get-calendar-event` — 查询日程 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `get-calendar-event-instances` — 查询日程实例 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/event_instances`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `get-calendar-event-instances-list` — 查询日程实例列表 (`GET` `/v7/calendars/{calendar_id}/event_instances`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `get-calendar-event-list` — 查询日程列表 (`GET` `/v7/calendars/{calendar_id}/events`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `get-calendar-id` — 查询日历 (`GET` `/v7/calendars/{calendar_id}`; scopes: `kso.calendar.read, kso.calendar.readwrite, kso.calendar.read, kso.calendar.readwrite`)
  - `get-calendar-list` — 查询日历列表 (`GET` `/v7/calendars`; scopes: `kso.calendar.read, kso.calendar.readwrite, kso.calendar.read, kso.calendar.readwrite`)
  - `get-calendar-permission-list` — 查询日历权限列表 (`GET` `/v7/calendars/{calendar_id}/permissions`; scopes: `kso.calendar.read, kso.calendar.readwrite, kso.calendar.read, kso.calendar.readwrite`)
  - `get-event-attendee-group-members` — 查询日程参与者为用户组的成员 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/attendee_groups/{group_id}/members`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `get-event-attendee-list` — 查询日程参与者列表 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/attendees`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `get-event-meeting-room-list` — 查询日程会议室列表 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/meeting_rooms`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `get-main-calendar` — 查询主日历信息 (`GET` `/v7/calendars/primary`; scopes: `kso.calendar.read, kso.calendar.readwrite, kso.calendar.read, kso.calendar.readwrite`)
  - `search-calendar-event` — 搜索日程 (`GET` `/v7/calendars/{calendar_id}/events/search`; scopes: `kso.calendar_events.read, kso.calendar_events.readwrite, kso.calendar_events.read, kso.calendar_events.readwrite`)
  - `subscribe-calendar` — 订阅日历 (`GET` `/v7/calendars/{calendar_id}/subscribe`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `unsubscribe-calendar` — 取消订阅日历 (`GET` `/v7/calendars/{calendar_id}/unsubscribe`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `update-calendar` — 更新日历 (`GET` `/v7/calendars/{calendar_id}/update`; scopes: `kso.calendar.readwrite, kso.calendar.readwrite`)
  - `update-calendar-event` — 更新日程 (`GET` `/v7/calendars/{calendar_id}/events/{event_id}/update`; scopes: `kso.calendar_events.readwrite, kso.calendar_events.readwrite`)

## Discovering Commands

```bash
wpscli calendars --help
wpscli schema calendars
```
