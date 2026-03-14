---
name: recipe-schedule-meeting-with-contacts
version: 1.0.0
description: "通过 cookie 私有接口搜索联系人、查忙闲并创建会议。"
metadata:
  openclaw:
    category: "recipe"
    requires:
      bins: ["wpscli"]
      skills: ["wps-shared"]
    domain: "scheduling"
---

# 查人后排会（Cookie 场景）

通过 cookie 私有接口搜索联系人、查忙闲并创建会议。

- Services: `cookie_contacts`, `free_busy_list`, `meetings`
- Auth sequence: `cookie -> user -> user`

## Steps

1. 搜索参会人: `wpscli cookie_contacts search-users-cookie --auth-type cookie --query query=张三`
2. 查询忙闲: `wpscli free_busy_list get-free-busy-list --auth-type user --query start_time=<iso> --query end_time=<iso>`
3. 创建会议: `wpscli meetings create-meeting --auth-type user --body '{...participants...}'`

## Caution

cookie 接口为私有能力，稳定性不保证；失败时需重新抓取 wps_sid。
