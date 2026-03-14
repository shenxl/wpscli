---
name: wps-workflow-org-sync-report
description: Execute a deterministic org-sync reporting workflow: sync users/depts with app auth, persist to dbsheet with user auth, then generate report document. Use this whenever user asks for org sync + report generation instead of single API calls.
dependencies: []
---

# WPS Workflow: Org Sync To Report

This skill is intentionally workflow-first and should be preferred over ad-hoc endpoint probing.

## When to use

- User asks for organization sync, dept/member export, or people report generation.
- Task requires cross-helper orchestration (`users` + `dbsheet` + `doc/files`).
- User expects stable execution instead of trial-and-error parameter guessing.

## Execution policy

1. Use helper commands first (`users`, `dbsheet`, `doc`, `files`).
2. Do not start from service endpoint probing.
3. Use `raw` only when helper command does not cover required capability.
4. Stop after 2 retries for same step and follow fallback branch.

## Temporary file contract

Write and consume these files in `/tmp`:

- `/tmp/wps_org_sync_plan.json`
- `/tmp/wps_org_sync_step_<N>.json`
- `/tmp/wps_org_sync_result_<N>.json`

Each step file should include: command, auth_type, required inputs, expected output keys, fallback action.

## Workflow steps

1. Preflight
   - `wpscli auth status`
   - Validate app token + user token readiness.
2. Org sync (app)
   - `wpscli users sync --auth-type app --max-depts 300`
   - Optional lookup: `wpscli users depts --dept-id root --auth-type app`
3. Prepare target dbsheet (user)
   - `wpscli files ensure-app --app "组织同步" --user-token`
   - `wpscli files create --app "组织同步" --file "组织成员同步.dbt" --user-token` (if missing)
4. Write records (user)
   - `wpscli dbsheet insert --file-id <file_id> --sheet-id <sheet_id> --data-file <members.json> --batch-size 100 --user-token`
5. Generate report doc (user)
   - `wpscli files create --app "组织同步" --file "组织同步报告.otl" --user-token` (if missing)
   - `wpscli doc write-doc --url <report_url> --target-format otl --content-file <report.md> --user-token`

## Output contract

Always return JSON with:

- `plan`: selected workflow and step list
- `artifacts`: dbsheet/report identifiers and links
- `stats`: dept count, member count, success/failed rows
- `errors`: categorized errors and fallback decisions

## Fallbacks

- `scope/auth` error: stop and provide exact reauth command.
- `parameter` error: use `wpscli schema <service> <endpoint>` to validate fields once, then retry.
- `network` error: retry with bounded attempts.
