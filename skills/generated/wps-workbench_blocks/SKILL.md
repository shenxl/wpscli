---
name: wps-workbench_blocks
version: 1.0.0
description: "WPS OpenAPI service: workbench_blocks"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli workbench_blocks --help"
---

# workbench_blocks service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli workbench_blocks <endpoint> [flags]
```

## API Resources

### workbench_blocks

  - `update-block-content` — 更新小组件内容 (`GET` `/v7/workbench_blocks/{block_id}/update_content`; scopes: `kso.workbench_block.readwrite`)

## Discovering Commands

```bash
wpscli workbench_blocks --help
wpscli schema workbench_blocks
```
