---
name: wps-meetings
version: 1.0.0
description: "WPS OpenAPI service: meetings"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli meetings --help"
---

# meetings service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli meetings <endpoint> [flags]
```

## API Resources

### meetings

  - `end-meeting` — 结束会议 (`GET` `/v7/meetings/{meeting_id}/end`; scopes: `kso.meeting.readwrite`)
  - `get-meeting` — 获取会议详情 (`GET` `/v7/meetings/{meeting_id}`; scopes: `kso.meeting.read, kso.meeting.readwrite, kso.meeting.read, kso.meeting.readwrite`)
  - `get-minute-summary` — 获取纪要总结要点 (`GET` `/v7/meetings/{meeting_id}/minutes/{minute_id}/summary`; scopes: `kso.meeting_minutes_content.read, kso.meeting_minutes_content.read`)
  - `get-minute-transcript` — 获取纪要语音转写 (`GET` `/v7/meetings/{meeting_id}/minutes/{minute_id}/transcript`; scopes: `kso.meeting_minutes_content.read, kso.meeting_minutes_content.read`)
  - `get-minutes` — 获取指定会议的纪要列表 (`GET` `/v7/meetings/{meeting_id}/minutes`; scopes: `kso.meeting_minutes.read, kso.meeting_minutes.readwrite, kso.meeting_minutes.read, kso.meeting_minutes.readwrite`)
  - `get-participants` — 获取会议参会人列表 (`GET` `/v7/meetings/{meeting_id}/participants`; scopes: `kso.meeting.read, kso.meeting.readwrite, kso.meeting.read, kso.meeting.readwrite`)
  - `get-recording-chapters` — 获取录制的章节 (`GET` `/v7/meetings/{meeting_id}/recordings/{recording_id}/chapters`; scopes: `kso.meeting_recording_content.read, kso.meeting_recording_content.read`)
  - `get-recording-summary` — 获取录制总结要点 (`GET` `/v7/meetings/{meeting_id}/recordings/{recording_id}/summary`; scopes: `kso.meeting_recording_content.read, kso.meeting_recording_content.read`)
  - `get-recording-transcript` — 获取录制语音转写 (`GET` `/v7/meetings/{meeting_id}/recordings/{recording_id}/transcript`; scopes: `kso.meeting_recording_content.read, kso.meeting_recording_content.read`)
  - `get-recordings` — 获取指定会议的录制列表 (`GET` `/v7/meetings/{meeting_id}/recordings`; scopes: `kso.meeting_recording.read, kso.meeting_recording.read, kso.meeting_recording.readwrite`)
  - `invite-participants` — 邀请参会人 (`GET` `/v7/meetings/{meeting_id}/participants/invite`; scopes: `kso.meeting.readwrite`)
  - `list-meetings` — 获取会议列表 (`GET` `/v7/meetings`; scopes: `kso.meeting.read, kso.meeting.read`)
  - `remove-participants` — 移除参会人 (`GET` `/v7/meetings/{meeting_id}/participants/remove`; scopes: `kso.meeting.readwrite`)
  - `set-host` — 设置主持人 (`GET` `/v7/meetings/{meeting_id}/set_host`; scopes: `kso.meeting.readwrite, kso.meeting.readwrite`)
  - `start-recording` — 开始录制 (`GET` `/v7/meetings/{meeting_id}/recordings/start`; scopes: `kso.meeting_recording.readwrite`)
  - `stop-recording` — 停止录制 (`GET` `/v7/meetings/{meeting_id}/recordings/stop`; scopes: `kso.meeting_recording.readwrite`)

## Discovering Commands

```bash
wpscli meetings --help
wpscli schema meetings
```
