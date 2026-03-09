---
name: wps-id_convert
version: 1.0.0
description: "WPS OpenAPI service: id_convert"
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    cliHelp: "wpscli id_convert --help"
---

# id_convert service

> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.

```bash
wpscli id_convert <endpoint> [flags]
```

## API Resources

### id_convert

  - `id-convert` — ID转换 (`POST` `/v7/id_convert`; scopes: `-`)

## Discovering Commands

```bash
wpscli id_convert --help
wpscli schema id_convert
```
