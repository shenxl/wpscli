---
name: wps-drives
version: 1.0.0
description: "WPS OpenAPI service: drives"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli drives --help"
    auth_types: ["app", "user"]
---

# drives service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli drives <endpoint> [flags]
```

## API Resources

### drives

  - `batch-copy-file` — 批量复制文件（夹） (`GET` `/v7/drives/{drive_id}/files/batch_copy`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `batch-create-drive-permissions` — 盘批量授权 (`GET` `/v7/drives/{drive_id}/permissions/batch_create`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `batch-create-fail-permissions` — 批量授权（单个文件） (`GET` `/v7/drives/{drive_id}/files/{file_id}/permissions/batch_create`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `batch-delete-drive-permissions` — 盘批量移除授权 (`GET` `/v7/drives/{drive_id}/permissions/batch_delete`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `batch-delete-fail-permissions` — 批量移除授权（单个文件） (`GET` `/v7/drives/{drive_id}/files/{file_id}/permissions/batch_delete`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `batch-delete-files-folders` — 批量删除文件(夹) (`GET` `/v7/drives/{drive_id}/files/batch_delete`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `batch-download-files` — 批量文件下载 (`GET` `/v7/drives/{drive_id}/files/batch_download`; scopes: `-`; auth: `both`)
  - `batch-remove-file` — 批量移动文件（夹） (`GET` `/v7/drives/{drive_id}/files/batch_move`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `check-file-exist` — 检查文件名是否已存在 (`GET` `/v7/drives/{drive_id}/files/{parent_id}/check_name`; scopes: `-`; auth: `both`)
  - `close-drive-link` — 取消文件盘分享 (`GET` `/v7/drives/{drive_id}/close_link`; scopes: `kso.drive.readwrite, kso.drive.readwrite`; auth: `both`)
  - `close-link` — 取消文件分享 (`GET` `/v7/drives/{drive_id}/files/{file_id}/close_link`; scopes: `kso.file_link.readwrite, kso.file_link.readwrite`; auth: `both`)
  - `complete-upload-file` — 提交文件上传完成 (`GET` `/v7/drives/{drive_id}/files/{parent_id}/commit_upload`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `create-drive` — 新建驱动盘 (`GET` `/v7/drives/create`; scopes: `kso.drive.readwrite, kso.drive.readwrite`; auth: `both`)
  - `create-file` — 新建文件（夹） (`GET` `/v7/drives/{drive_id}/files/{parent_id}/create`; scopes: `kso.file.readwrite, kso.file.readwrite, kso.mcp.readwrite`; auth: `both`)
  - `create-role` — 新建文档权限角色 (`GET` `/v7/drives/{drive_id}/roles/create`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `delete-role` — 删除文档权限角色 (`GET` `/v7/drives/{drive_id}/roles/{role_id}/delete`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `get-drive` — 获取盘信息 (`GET` `/v7/drives/{drive_id}/meta`; scopes: `kso.drive.readwrite, kso.drive.readwrite`; auth: `both`)
  - `get-drive-list` — 获取盘列表 (`GET` `/v7/drives`; scopes: `kso.drive.readwrite, kso.drive.readwrite, kso.mcp.readwrite`; auth: `both`)
  - `get-file-content` — 文档内容抽取 (`GET` `/v7/drives/{drive_id}/files/{file_id}/content`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read, kso.mcp.readwrite`; auth: `both`)
  - `get-file-download` — 获取文件下载信息 (`GET` `/v7/drives/{drive_id}/files/{file_id}/download`; scopes: `kso.file.readwrite, kso.file.read, kso.appfile.readwrite, kso.file.readwrite, kso.file.read`; auth: `both`)
  - `get-file-info-by-id` — 根据drive_id和file_id获取文件信息 (`GET` `/v7/drives/{drive_id}/files/{file_id}/meta`; scopes: `kso.file.readwrite, kso.file.read, kso.appfile.readwrite, kso.file.readwrite, kso.file.read`; auth: `both`)
  - `get-file-list` — 获取子文件列表 (`GET` `/v7/drives/{drive_id}/files/{parent_id}/children`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read`; auth: `both`)
  - `get-file-path` — 获取文件路径 (`GET` `/v7/drives/{drive_id}/files/{file_id}/path`; scopes: `kso.file.readwrite, kso.file.read, kso.file.readwrite, kso.file.read`; auth: `both`)
  - `get-upload-file` — 请求文件上传信息 (`GET` `/v7/drives/{drive_id}/files/{parent_id}/request_upload`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `get-user-file-permissions` — 获取用户的文件操作权限 (`GET` `/v7/drives/{drive_id}/files/{file_id}/permissions/acl`; scopes: `kso.file_permission.readwrite, kso.file_permission.read`; auth: `both`)
  - `get-version` — 获取文件版本信息 (`GET` `/v7/drives/{drive_id}/files/{file_id}/versions/{version_num}/meta`; scopes: `kso.file_version.readwrite, kso.file_version.read, kso.file_version.readwrite, kso.file_version.read`; auth: `both`)
  - `get-version-download-url` — 获取指定文件版本下载地址 (`GET` `/v7/drives/{drive_id}/files/{file_id}/versions/{version_num}/download`; scopes: `kso.file_version.readwrite, kso.file_version.read, kso.file_version.readwrite, kso.file_version.read`; auth: `both`)
  - `get-version-list` — 获取文件版本列表 (`GET` `/v7/drives/{drive_id}/files/{file_id}/versions`; scopes: `kso.file_version.readwrite, kso.file_version.read, kso.file_version.readwrite, kso.file_version.read`; auth: `both`)
  - `init-multipart-upload-task` — 分块上传任务初始化 (`GET` `/v7/drives/{drive_id}/files/{parent_id}/create_multipart_upload_task`; scopes: `kso.file.readwrite, kso.appfile.readwrite, kso.file.readwrite`; auth: `both`)
  - `list-file-permissions` — 列举文件权限列表 (`GET` `/v7/drives/{drive_id}/files/{file_id}/permissions`; scopes: `kso.file_permission.readwrite, kso.file_permission.read`; auth: `both`)
  - `open-drive-link` — 开启文件盘分享 (`GET` `/v7/drives/{drive_id}/open_link`; scopes: `kso.drive.readwrite, kso.drive.readwrite`; auth: `both`)
  - `open-link` — 开启文件分享 (`GET` `/v7/drives/{drive_id}/files/{file_id}/open_link`; scopes: `kso.file_link.readwrite, kso.file_link.readwrite, kso.mcp.readwrite`; auth: `both`)
  - `rapid-upload` — 文件秒传 (`GET` `/v7/drives/{drive_id}/files/{parent_id}/rapid_upload`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `rename-file` — 重命名文件（夹） (`GET` `/v7/drives/{drive_id}/files/{file_id}/rename`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `roles` — 获取文档权限角色列表 (`GET` `/v7/drives/{drive_id}/roles`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite, kso.file_permission.read`; auth: `both`)
  - `save-as-file` — 文件另存为 (`GET` `/v7/drives/{drive_id}/files/{file_id}/save_as`; scopes: `-`; auth: `both`)
  - `transfer-file` — 转让文件拥有者 (`GET` `/v7/drives/{drive_id}/files/{file_id}/transfer_owner`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `update-file` — 修改文件元数据信息 (`GET` `/v7/drives/{drive_id}/files/{file_id}/update`; scopes: `kso.file.readwrite, kso.file.readwrite`; auth: `both`)
  - `update-role` — 更新文档权限角色 (`GET` `/v7/drives/{drive_id}/roles/{role_id}/update`; scopes: `kso.file_permission.readwrite, kso.file_permission.readwrite`; auth: `both`)
  - `update-version` — 更新文件版本信息 (`GET` `/v7/drives/{drive_id}/files/{file_id}/versions/{version_num}/update`; scopes: `kso.file_version.readwrite, kso.file_version.readwrite`; auth: `both`)

## Discovering Commands

```bash
wpscli drives --help
wpscli schema drives
```
