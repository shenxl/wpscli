---
name: wps-mail_contacts
version: 1.0.0
description: "WPS OpenAPI service: mail_contacts"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli mail_contacts --help"
    auth_types: ["app"]
---

# mail_contacts service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli mail_contacts <endpoint> [flags]
```

## API Resources

### mail_contacts

  - `batch-create-mail-contacts` — 批量新增邮箱联系人 (`GET` `/v7/mail_contacts/batch_create`; scopes: `kso.mail_contact.readwrite`; auth: `app`)
  - `batch-create-mail-contacts-members` — 批量创建邮箱联系人成员 (`GET` `/v7/mail_contacts/{id}/members/batch_create`; scopes: `kso.mail_contact.readwrite`; auth: `app`)
  - `batch-delete-mail-contacts` — 批量删除邮箱联系人 (`GET` `/v7/mail_contacts/batch_delete`; scopes: `kso.mail_contact.readwrite`; auth: `app`)
  - `batch-delete-mail-contacts-members` — 批量删除邮箱联系人成员 (`GET` `/v7/mail_contacts/{id}/members/batch_delete`; scopes: `kso.mail_contact.readwrite`; auth: `app`)
  - `batch-update-mail-contacts` — 批量修改邮箱联系人 (`GET` `/v7/mail_contacts/batch_update`; scopes: `kso.mail_contact.readwrite`; auth: `app`)
  - `create-mail-contacts` — 新增邮箱联系人 (`GET` `/v7/mail_contacts`; scopes: `kso.mail_contact.readwrite`; auth: `app`)
  - `delete-mail-contacts` — 删除邮箱联系人 (`GET` `/v7/mail_contacts/{id}/delete`; scopes: `kso.mail_contact.readwrite`; auth: `app`)
  - `get-list-mail-contacts` — 获取邮箱联系人列表 (`GET` `/v7/mail_contacts`; scopes: `kso.mail_contact.readwrite, kso.mail_contact.read`; auth: `app`)
  - `get-mail-contacts-members` — 获取邮箱联系人成员列表 (`GET` `/v7/mail_contacts/{id}/members`; scopes: `kso.mail_contact.readwrite, kso.mail_contact.read`; auth: `app`)
  - `update-mail-contacts` — 修改邮箱联系人信息 (`GET` `/v7/mail_contacts/{id}/update`; scopes: `kso.mail_contact.readwrite`; auth: `app`)

## Discovering Commands

```bash
wpscli mail_contacts --help
wpscli schema mail_contacts
```
