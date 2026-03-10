---
name: wps-skill-builder
description: Build reusable Claude skills from WPS APIs by discovering endpoints through wpscli schema and generating Skill.md plus reference assets.
dependencies: []
---

# WPS Skill Builder

Use this skill when you need to create a new reusable skill package from WPS OpenAPI capabilities.

It is designed around `wpscli`, so discovery and generation are both grounded in the same executable API surface that agents actually call.

## What It Produces

- `Skill.md` (Claude-compatible frontmatter: `name`, `description`, optional `dependencies`)
- `REFERENCE.md` (selected endpoint inventory with method/path/scopes and runnable command templates)
- `commands.json` (machine-readable command metadata for downstream automation)
- `manifest.json` (generation metadata)

## When To Use

- You need a new skill for one WPS domain (for example `drives`, `depts`, `dbsheet`).
- You want a filtered subset of endpoints for a focused workflow.
- You need a reproducible generation flow that can be rerun after API changes.

## Generator Entry

Run from this folder:

```bash
python3 build_skill.py create \
  --name wps-drives-readonly \
  --description "Read WPS drive metadata and files for analysis workflows." \
  --service drives \
  --include list meta search \
  --limit 12 \
  --output-dir ./dist
```

Inspect only (no files generated):

```bash
python3 build_skill.py inspect \
  --service drives \
  --include list search \
  --limit 8
```

## Input Contract

- Required:
  - `--name`: target skill package name (directory name)
  - `--description`: short intent and trigger condition
  - `--service`: one `wpscli` service name
- Optional:
  - `--include`: endpoint keyword filters (repeatable)
  - `--exclude`: endpoint keyword exclusions (repeatable)
  - `--limit`: max endpoint count
  - `--auth-type`: `app` or `user` for generated command templates
  - `--output-dir`: destination root

## Quality Checklist

After generation, verify:

1. Frontmatter has valid `name` and `description`.
2. `REFERENCE.md` only contains endpoints relevant to the workflow.
3. Generated command templates align with actual auth mode and scope expectations.
4. The package folder can be zipped directly for Claude custom skill upload.

## Notes

- This generator uses `wpscli schema <service> --output json` as the source of truth.
- It follows a pattern similar to `google-cli` `generate-skills`: generate docs from CLI metadata, not from hand-maintained static text.
