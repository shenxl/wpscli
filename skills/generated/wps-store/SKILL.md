---
name: wps-store
version: 1.0.0
description: "WPS OpenAPI service: store"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli store --help"
---

# store service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli store <endpoint> [flags]
```

## API Resources

### store

  - `batch-create-allocations` — 批量创建授权分配记录 (`POST` `/v7/store/allocations/batch_create`; scopes: `kso.store_allocation.readwrite.all, kso.store_allocation.readwrite.ownedby`)
  - `batch-delete-allocations` — 批量取消授权分配记录 (`POST` `/v7/store/allocations/batch_delete`; scopes: `kso.store_allocation.readwrite.all, kso.store_allocation.readwrite.ownedby`)
  - `create-orders` — 创建商城订单 (`POST` `/v7/store/orders/create`; scopes: `kso.store_order.readwrite`)
  - `get-allocations` — 获取商品授权分配记录列表 (`GET` `/v7/store/allocations`; scopes: `kso.store_allocation.readwrite.all, kso.store_allocation.readwrite.ownedby, kso.store_allocation.read.all, kso.store_allocation.read.ownedby`)
  - `get-entitlements` — 获取商品授权列表 (`GET` `/v7/store/entitlements`; scopes: `kso.store_allocation.readwrite.all, kso.store_allocation.readwrite.ownedby, kso.store_allocation.read.all, kso.store_allocation.read.ownedby`)
  - `get-orders-info` — 获取商城订单详情 (`GET` `/v7/store/orders/{order_id}`; scopes: `kso.store_order.readwrite.all, kso.store_order.readwrite.ownedby, kso.store_order.read.all, kso.store_order.read.ownedby`)
  - `get-orders-list` — 获取商城订单列表 (`GET` `/v7/store/orders`; scopes: `kso.store_order.readwrite.all, kso.store_order.readwrite.ownedby, kso.store_order.read.all, kso.store_order.read.ownedby`)

## Discovering Commands

```bash
wpscli store --help
wpscli schema store
```
