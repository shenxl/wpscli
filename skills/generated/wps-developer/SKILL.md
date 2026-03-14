---
name: wps-developer
version: 1.0.0
description: "WPS OpenAPI service: developer"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli developer --help"
    auth_types: ["app", "user"]
---

# developer service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli developer <endpoint> [flags]
```

## API Resources

### developer

  - `app-admin-check` — 校验应用管理员 (`GET` `/v7/developer/app_admins/{user_id}/check`; scopes: `kso.app_admin.read`; auth: `app`)
  - `app-admin-list` — 查询应用管理员列表 (`GET` `/v7/developer/app_admins`; scopes: `kso.app_admin.read`; auth: `app`)
  - `app-admin-scope` — 获取应用管理员管理范围 (`GET` `/v7/developer/get_app_admin_scope`; scopes: `kso.app_admin.read`; auth: `app`)
  - `app-member-change-owner` — 转让应用所有者 (`GET` `/v7/developer/apps/{app_id}/change_owner`; scopes: `kso.apps.readwrite, kso.apps.readwrite`; auth: `both`)
  - `app-member-list` — 获取应用成员列表 (`GET` `/v7/developer/apps/{app_id}/members`; scopes: `kso.apps.read, kso.apps.read`; auth: `both`)
  - `app-member-update` — 更新应用成员 (`GET` `/v7/developer/apps/{app_id}/members/batch_update`; scopes: `kso.apps.readwrite, kso.apps.readwrite`; auth: `both`)
  - `app-subject-add` — 新增企业应用可用范围 (`GET` `/v7/developer/apps/{app_id}/subjects/batch_add`; scopes: `kso.app_subject.readwrite`; auth: `both`)
  - `app-subject-remove` — 删除企业应用可用范围 (`GET` `/v7/developer/apps/{app_id}/subjects/batch_remove`; scopes: `kso.app_subject.readwrite`; auth: `both`)
  - `app-version-list` — 获取应用版本列表 (`GET` `/v7/developer/apps/{app_id}/versions`; scopes: `kso.app_version.read`; auth: `app`)
  - `batch-update` — 批量更新应用角标 (`GET` `/v7/developer/app_badges/batch_update`; scopes: `kso.apps.readwrite`; auth: `both`)
  - `disable-app` — 停用应用 (`GET` `/v7/developer/apps/{app_id}/disable`; scopes: `kso.apps.readwrite`; auth: `both`)
  - `enable-app` — 启用应用 (`GET` `/v7/developer/apps/{app_id}/enable`; scopes: `kso.apps.readwrite`; auth: `both`)
  - `get-app-contacts-range` — 获取应用的通讯录权限范围 (`GET` `/v7/developer/apps/{app_id}/contacts_range`; scopes: `kso.app_version.read, kso.app_version.readwrite`; auth: `app`)
  - `get-app-contacts-range-apply` — 获取应用版本中开发者申请的通讯录权限范围 (`GET` `/v7/developer/apps/{app_id}/contacts_range_apply`; scopes: `kso.app_version.read, kso.app_version.readwrite`; auth: `app`)
  - `get-app-version-detail` — 获取应用版本信息 (`GET` `/v7/developer/apps/{app_id}/versions/{version}`; scopes: `kso.app_version.read`; auth: `app`)
  - `get-detail` — 获取指定应用详情 (`GET` `/v7/developer/apps/{app_id}`; scopes: `kso.app_version.read, kso.app_version.readwrite`; auth: `app`)
  - `get-info` — 获取应用信息 (`GET` `/v7/developer/applications/current`; scopes: `-`; auth: `both`)
  - `get-visibility` — 获取应用在企业内的可用范围 (`GET` `/v7/developer/applications/current/subjects`; scopes: `kso.app.read`; auth: `both`)
  - `get-visible-apps` — 获取用户可用的应用 (`GET` `/v7/developer/visible_apps`; scopes: `kso.apps.read`; auth: `both`)
  - `list-installed-apps` — 获取企业安装应用列表 (`GET` `/v7/developer/apps`; scopes: `kso.app_version.read, kso.app_version.readwrite`; auth: `app`)
  - `update-app-approval-status` — 更新应用审核状态 (`GET` `/v7/developer/apps/{app_id}/update_approval_status`; scopes: `kso.app_version.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli developer --help
wpscli schema developer
```
