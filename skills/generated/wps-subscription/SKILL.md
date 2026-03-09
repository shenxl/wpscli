---
name: wps-subscription
version: 1.0.0
description: "WPS OpenAPI service: subscription"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli subscription --help"
---

# subscription service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli subscription <endpoint> [flags]
```

## API Resources

### subscription

  - `get-subscription-account-storage` — 获取账号文件下载链接 (`GET` `/v7/subscription/accounts/{account_id}/storages/{store_key}`; scopes: `kso.subscription_account.read`)
  - `get-subscription-accounts` — 获取账号列表 (`GET` `/v7/subscription/accounts`; scopes: `kso.subscription_account.read`)
  - `get-subscription-content-storage` — 获取内容文件下载链接 (`GET` `/v7/subscription/contents/{content_id}/storages/{store_key}`; scopes: `kso.subscription_content.read`)
  - `get-subscription-published-content` — 获取已发布内容列表 (`GET` `/v7/subscription/accounts/{account_id}/published_contents`; scopes: `kso.subscription_content.read`)

## Discovering Commands

```bash
wpscli subscription --help
wpscli schema subscription
```
