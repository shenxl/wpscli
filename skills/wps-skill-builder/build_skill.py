#!/usr/bin/env python3
"""
Skill creator for WPS:
- Discover all API capabilities from descriptors / wpscli schema
- Compose business skills from helper workflows (files/doc/users/dbsheet...)
- Generate Claude-compatible skill package + eval scaffolding
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple


HELPER_PROFILES: Dict[str, Dict[str, Any]] = {
    "files": {
        "summary": "应用目录与文件编排（创建、上传、下载、状态追踪）",
        "services": ["drives", "files", "applications", "multipart_upload_tasks"],
        "commands": [
            "wpscli files ensure-app --app <APP> --user-token",
            "wpscli files create --app <APP> --file <FILE> --user-token",
            "wpscli files upload --app <APP> --file <LOCAL_FILE> --user-token",
            "wpscli files transfer --mode upload --app <APP> --file <LOCAL_FILE> --user-token",
        ],
    },
    "doc": {
        "summary": "文档统一读写（otl/doc/pdf/xlsx/dbt 路由）",
        "services": ["drives", "documents", "sheets", "aidocs", "dbsheet", "coop_dbsheet"],
        "commands": [
            "wpscli doc read-doc --url <URL> --user-token",
            "wpscli doc write-doc --url <URL> --content <CONTENT> --user-token",
            "wpscli doc read-doc --url <URL> --export-parquet <PATH> --user-token",
        ],
    },
    "dbsheet": {
        "summary": "多维表 SQL-like 能力（init/select/insert/update/delete）",
        "services": ["dbsheet", "coop_dbsheet", "drives"],
        "commands": [
            "wpscli dbsheet init --url <URL> --schema-file <SCHEMA.yaml> --user-token",
            "wpscli dbsheet select --url <URL> --where \"状态 = '进行中'\" --user-token",
            "wpscli dbsheet insert --url <URL> --values '[{\"字段\":\"值\"}]' --user-token",
        ],
    },
    "users": {
        "summary": "组织通讯录查询与缓存（app token）",
        "services": ["users", "depts", "contacts", "groups"],
        "commands": [
            "wpscli users sync --auth-type app --max-depts 300",
            "wpscli users find --name <姓名关键字> --auth-type app",
            "wpscli users members --dept-id <DEPT_ID> --recursive true --auth-type app",
        ],
    },
    "sheets": {
        "summary": "表格读写与表格化输出",
        "services": ["sheets", "drives"],
        "commands": [
            "wpscli doc read-doc --url <XLSX_URL> --xlsx-row-head 100 --user-token",
            "wpscli doc write-doc --url <XLSX_URL> --target-format xlsx --content <JSON_2D> --user-token",
        ],
    },
    "airpage": {
        "summary": "智能文档块（otl）写入",
        "services": ["documents", "aidocs", "drives"],
        "commands": [
            "wpscli airpage write-markdown --url <URL> --content <MARKDOWN> --user-token",
            "wpscli doc write-doc --url <URL> --target-format otl --content <MARKDOWN> --user-token",
        ],
    },
    "calendar": {
        "summary": "日历事件编排",
        "services": ["calendars", "meetings"],
        "commands": [
            "wpscli calendar list --user-token",
            "wpscli calendar create-event --calendar-id <ID> --json <EVENT_JSON> --user-token",
        ],
    },
    "chat": {
        "summary": "会话与消息发送",
        "services": ["chats", "messages"],
        "commands": [
            "wpscli chat send --chat-id <CHAT_ID> --text <TEXT> --user-token",
            "wpscli raw POST /v7/messages/create --user-token --body '{\"chat_id\":\"<ID>\",\"text\":\"hello\"}'",
        ],
    },
    "meeting": {
        "summary": "会议与会议室操作",
        "services": ["meetings", "meeting_rooms", "meeting_room_bookings"],
        "commands": [
            "wpscli meeting list --user-token",
            "wpscli meeting create --json <MEETING_JSON> --user-token",
        ],
    },
}

HELPER_KEYWORDS: Dict[str, List[str]] = {
    "files": ["文件", "目录", "上传", "下载", "drive", "cloud", "应用目录"],
    "doc": ["文档", "read", "write", "pdf", "ppt", "docx", "otl", "dbt"],
    "dbsheet": ["多维表", "dbsheet", "记录", "sql", "crud", "表记录"],
    "users": ["用户", "部门", "通讯录", "组织", "成员", "员工"],
    "sheets": ["xlsx", "sheet", "表格", "excel", "单元格"],
    "airpage": ["airpage", "智能文档块", "块", "markdown", "otl"],
    "calendar": ["日历", "事件", "排期", "会议时间"],
    "chat": ["消息", "会话", "群", "chat", "通知"],
    "meeting": ["会议", "会议室", "meeting"],
}


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _descriptor_dir() -> Path:
    return _repo_root() / "descriptors"


_SKILL_SYNC_DONE = False


def _generated_skills_dir() -> Path:
    return _repo_root() / "skills" / "generated"


def _ensure_generated_skills(wpscli_bin: str) -> None:
    """Keep builder and Rust skill pipeline aligned by invoking `wpscli generate-skills`."""
    global _SKILL_SYNC_DONE
    if _SKILL_SYNC_DONE:
        return
    out_dir = _generated_skills_dir()
    proc = subprocess.run(
        [wpscli_bin, "generate-skills", "--out-dir", str(out_dir), "--output", "json"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"failed to sync generated skills via wpscli generate-skills\n"
            f"command: {wpscli_bin} generate-skills --out-dir {out_dir}\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )
    _SKILL_SYNC_DONE = True


def _run_wpscli(wpscli_bin: str, args: List[str]) -> Dict[str, Any]:
    proc = subprocess.run(
        [wpscli_bin, *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"wpscli command failed: {wpscli_bin} {' '.join(args)}\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(
            f"failed to parse wpscli output as JSON for command: {wpscli_bin} {' '.join(args)}\n"
            f"output:\n{proc.stdout}"
        ) from exc


def _load_service_schema(wpscli_bin: str, service: str) -> Dict[str, Any]:
    payload = _run_wpscli(wpscli_bin, ["schema", service, "--output", "json"])
    if not isinstance(payload, dict) or "endpoints" not in payload:
        raise RuntimeError(f"unexpected schema payload for service '{service}'")
    return payload


def _load_all_endpoints() -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    for fp in sorted(_descriptor_dir().glob("*.json")):
        try:
            raw = json.loads(fp.read_text(encoding="utf-8"))
        except Exception:
            continue
        if not isinstance(raw, dict):
            continue
        service = str(raw.get("service", "")).strip()
        endpoints = raw.get("endpoints")
        if not service or not isinstance(endpoints, list):
            continue
        for ep in endpoints:
            if not isinstance(ep, dict):
                continue
            record = dict(ep)
            record["service"] = service
            out.append(record)
    return out


def _normalize_name(name: str) -> str:
    value = name.strip().lower()
    value = re.sub(r"[^a-z0-9_-]+", "-", value)
    value = re.sub(r"-{2,}", "-", value).strip("-")
    if not value:
        raise ValueError("skill name is empty after normalization")
    return value


def _uniq(items: List[str]) -> List[str]:
    seen = set()
    out: List[str] = []
    for it in items:
        v = it.strip()
        if not v or v in seen:
            continue
        seen.add(v)
        out.append(v)
    return out


def _text_blob(ep: Dict[str, Any]) -> str:
    fields = [
        str(ep.get("service", "")),
        str(ep.get("id", "")),
        str(ep.get("name", "")),
        str(ep.get("summary", "")),
        str(ep.get("path", "")),
    ]
    return " ".join(fields).lower()


def _query_terms(query: str) -> List[str]:
    q = query.strip().lower()
    if not q:
        return []
    terms = [x for x in re.split(r"[\s,;/|]+", q) if x]
    if not terms:
        return [q]
    if len(terms) == 1:
        token = terms[0]
        # For Chinese long phrases, add bi-grams to improve matching recall.
        if re.search(r"[\u4e00-\u9fff]", token) and len(token) >= 4:
            grams = [token[i : i + 2] for i in range(len(token) - 1)]
            terms.extend(grams[:24])
    return _uniq(terms)


def _required_path_params(ep: Dict[str, Any]) -> List[str]:
    params = ep.get("params", {})
    path_params = params.get("path", []) if isinstance(params, dict) else []
    required: List[str] = []
    for p in path_params:
        if not isinstance(p, dict):
            continue
        name = str(p.get("name", "")).strip()
        req = bool(p.get("required"))
        if name and req:
            required.append(name)
    return required


def _command_template(service: str, ep: Dict[str, Any], auth_type: str) -> str:
    cmd = ["wpscli", service, str(ep.get("id", ""))]
    for p in _required_path_params(ep):
        cmd.extend(["--path-param", f"{p}=<{p}>"])
    method = str(ep.get("http_method", "GET")).upper()
    if method in {"POST", "PUT", "PATCH", "DELETE"}:
        cmd.extend(["--body", "'{}'"])
    cmd.extend(["--auth-type", auth_type])
    return " ".join(cmd)


def _match_endpoint(
    ep: Dict[str, Any],
    query_terms: List[str],
    include_terms: List[str],
    exclude_terms: List[str],
    service_allow: List[str],
) -> Tuple[bool, int]:
    blob = _text_blob(ep)
    service = str(ep.get("service", ""))
    if service_allow and service not in service_allow:
        return False, 0
    if include_terms and not any(t in blob for t in include_terms):
        return False, 0
    if exclude_terms and any(t in blob for t in exclude_terms):
        return False, 0
    if query_terms and not any(t in blob for t in query_terms):
        return False, 0

    score = 0
    score += sum(2 for t in query_terms if t in blob)
    score += sum(1 for t in include_terms if t in blob)
    score += 1 if str(ep.get("summary", "")).strip() else 0
    return True, score


def _discover(
    endpoints: List[Dict[str, Any]],
    query: str,
    includes: List[str],
    excludes: List[str],
    services: List[str],
    limit: int,
) -> List[Dict[str, Any]]:
    query_terms = _query_terms(query)
    include_terms = [x.lower() for x in includes if x.strip()]
    exclude_terms = [x.lower() for x in excludes if x.strip()]
    service_allow = _uniq(services)

    scored: List[Tuple[int, Dict[str, Any]]] = []
    for ep in endpoints:
        ok, score = _match_endpoint(ep, query_terms, include_terms, exclude_terms, service_allow)
        if ok:
            scored.append((score, ep))

    scored.sort(key=lambda it: (-it[0], str(it[1].get("service", "")), str(it[1].get("id", ""))))
    selected = [ep for _, ep in scored]
    if limit > 0:
        selected = selected[:limit]
    return selected


def _infer_helpers(goal: str) -> List[str]:
    text = goal.lower()
    out: List[str] = []
    for helper, kws in HELPER_KEYWORDS.items():
        if any(kw.lower() in text for kw in kws):
            out.append(helper)
    return _uniq(out)


def _helper_services(helpers: List[str]) -> List[str]:
    out: List[str] = []
    for h in helpers:
        profile = HELPER_PROFILES.get(h, {})
        out.extend(profile.get("services", []))
    return _uniq(out)


def _render_api_reference(endpoints: List[Dict[str, Any]], auth_type: str) -> str:
    lines = [
        "# API Baseline (Discovered)",
        "",
        "| service | endpoint | method | path | scopes | command |",
        "|---|---|---|---|---|---|",
    ]
    for ep in endpoints:
        service = str(ep.get("service", ""))
        endpoint_id = str(ep.get("id", ""))
        method = str(ep.get("http_method", "GET")).upper()
        path = str(ep.get("path", ""))
        scopes = ", ".join(ep.get("scopes", []) or [])
        cmd = _command_template(service, ep, auth_type)
        lines.append(
            f"| `{service}` | `{endpoint_id}` | `{method}` | `{path}` | `{scopes or '-'}` | `{cmd}` |"
        )
    lines.append("")
    return "\n".join(lines)


def _render_helper_reference(helpers: List[str]) -> str:
    lines = ["# Helper Composition", ""]
    for h in helpers:
        profile = HELPER_PROFILES.get(h)
        if not profile:
            continue
        lines.extend(
            [
                f"## {h}",
                "",
                f"- Summary: {profile['summary']}",
                f"- Service hints: {', '.join(profile.get('services', []))}",
                "",
                "Command patterns:",
            ]
        )
        for cmd in profile.get("commands", []):
            lines.append(f"- `{cmd}`")
        lines.append("")
    return "\n".join(lines)


def _render_workflow_guardrails(name: str, goal: str, helpers: List[str]) -> str:
    helper_hint = ", ".join(helpers) if helpers else "files, doc, users"
    return "\n".join(
        [
            "# Workflow Guardrails",
            "",
            "Use these rules to keep execution deterministic and reduce trial-and-error.",
            "",
            "## 1) Workflow-first policy",
            "",
            "- Prefer helper/recipe composition before single endpoint calls.",
            "- Use `wpscli raw` only when helper/recipe path is clearly unavailable.",
            f"- For this skill (`{name}`), start from helpers: `{helper_hint}`.",
            "",
            "## 2) Temporary file contract (/tmp)",
            "",
            "Before executing any command, write structured files:",
            "",
            f"- Plan file: `/tmp/{name}_plan.json`",
            f"- Step params: `/tmp/{name}_step_<N>.json`",
            f"- Step result: `/tmp/{name}_result_<N>.json`",
            "",
            "Each `step_<N>.json` should include:",
            "",
            "```json",
            "{",
            '  "step": 1,',
            '  "intent": "what this step does",',
            '  "command": "wpscli <helper> <cmd> ...",',
            '  "auth_type": "app|user|cookie",',
            '  "inputs": {},',
            '  "expected_keys": ["ok", "data"],',
            '  "fallback": "what to do on failure"',
            "}",
            "```",
            "",
            "## 3) Retry / recovery policy",
            "",
            "- Retry same command at most 2 times.",
            "- If failure repeats, switch to predefined fallback branch.",
            "- Return structured error class: `auth`, `scope`, `parameter`, `network`, `business`.",
            "",
            "## 4) Hard stops",
            "",
            "- Do not keep mutating parameters without a schema-based reason.",
            "- For write/delete actions, run `--dry-run` first when possible.",
            "- If required identifiers are missing, stop and request exact input.",
            "",
            "## 5) Goal reminder",
            "",
            f"- Target business goal: {goal}",
            "",
        ]
    )


def _build_eval_prompts(skill_name: str, goal: str, helpers: List[str]) -> List[Dict[str, Any]]:
    helper_hint = "、".join(helpers) if helpers else "files/doc/users"
    return [
        {
            "id": 1,
            "prompt": f"请基于目标“{goal}”执行一次完整业务流程，优先使用 {helper_hint} 相关命令，输出结构化 JSON 执行报告。",
            "expected_output": "给出分步骤执行日志、关键资源 ID、结果摘要和下一步建议。",
            "files": [],
        },
        {
            "id": 2,
            "prompt": f"模拟权限不足或参数缺失场景，使用技能 {skill_name} 输出可恢复的错误分类和重试建议。",
            "expected_output": "错误类别明确（auth/parameter/permission/retryable），包含可执行修复命令。",
            "files": [],
        },
        {
            "id": 3,
            "prompt": f"在同一任务中串联至少两个 helper（例如 files + doc 或 users + dbsheet）完成“{goal}”的编排。",
            "expected_output": "明确显示跨 helper 的输入输出衔接与最终产物。",
            "files": [],
        },
    ]


def _render_business_skill_md(
    name: str,
    description: str,
    goal: str,
    helpers: List[str],
    endpoints: List[Dict[str, Any]],
    auth_type: str,
) -> str:
    helper_list = ", ".join(helpers) if helpers else "files, doc, users"
    service_list = ", ".join(_uniq([str(ep.get("service", "")) for ep in endpoints]))
    lines: List[str] = [
        "---",
        f"name: {name}",
        f"description: {description}",
        "dependencies: []",
        "---",
        "",
        f"# {name}",
        "",
        "This is a business-orchestration skill package generated by `wps-skill-builder`.",
        "It is intentionally pushy on trigger conditions: if user intent overlaps this workflow, use this skill.",
        "",
        "## Intent",
        "",
        f"- Goal: {goal}",
        f"- Helper composition: {helper_list}",
        f"- API baseline services: {service_list or '-'}",
        f"- Suggested auth mode for templates: `{auth_type}`",
        "",
        "## Triggering Guidance (Important)",
        "",
        "Use this skill whenever user requests imply a multi-step WPS business workflow,",
        "especially when tasks require chaining create/read/write/search/sync actions instead of single API calls.",
        "",
        "## Workflow-First Operating Mode",
        "",
        "Do NOT default to low-level endpoint probing. Use this strict order:",
        "1. Match an existing recipe/workflow pattern first.",
        "2. Compose helper commands (`files/doc/users/dbsheet/...`).",
        "3. Use service endpoints only for missing helper capability.",
        "4. Use `raw` as last resort with explicit rationale.",
        "",
        "If a step can be executed through a helper command, do not replace it with raw endpoint calls.",
        "",
        "## /tmp Execution Contract",
        "",
        "Claude Code often materializes parameters into temp files. Reuse that behavior intentionally:",
        f"- Write a plan to `/tmp/{name}_plan.json` before execution.",
        f"- Write each step's params to `/tmp/{name}_step_<N>.json`.",
        f"- Save each step's result to `/tmp/{name}_result_<N>.json`.",
        "This keeps runs reproducible and makes retries deterministic.",
        "",
        "## Execution Loop",
        "",
        "1. Capture user intent, constraints, and expected output format.",
        "2. Choose helper workflow first, then map required APIs from `references/API_BASELINE.md`.",
        "3. Confirm auth/scope readiness (`wpscli auth status`, `wpscli doctor`).",
        "4. Run workflow step-by-step with stable JSON outputs.",
        "5. If failure occurs, classify and provide retryable recovery actions.",
        "6. Persist key artifacts and provide summary for downstream agent steps.",
        "",
        "## Workflow Blueprint",
        "",
        "### Phase A: Discover",
        "- Inspect helper commands in `references/HELPER_COMPOSITION.md`.",
        "- Match required endpoints in `references/API_BASELINE.md`.",
        "",
        "### Phase B: Compose",
        "- Build a chain of helper commands for business intent.",
        "- Prefer high-level helper commands before raw endpoint calls.",
        "",
        "### Phase C: Run",
        "- Execute commands with deterministic parameters.",
        "- Keep intermediate outputs structured and traceable.",
        "",
        "### Phase D: Verify and Recover",
        "- Verify expected result fields and business invariants.",
        "- On failure, return category + suggested_action + retry hint.",
        "",
        "## Output Contract",
        "",
        "Always return:",
        "- `plan`: resolved workflow steps",
        "- `actions`: executed commands and key parameters",
        "- `artifacts`: created/updated resources",
        "- `result`: final business result",
        "- `errors`: categorized issues and recovery hints",
        "",
        "## Evaluation",
        "",
        "- Seed eval prompts: `evals/evals.json`",
        "- Workspace convention: `workspace/iteration-N/`",
        "- Compare with/without skill execution quality before revising skill rules.",
        "",
        "## References",
        "",
        "- `references/HELPER_COMPOSITION.md`",
        "- `references/API_BASELINE.md`",
        "- `references/WORKFLOW_GUARDRAILS.md`",
        "- `commands.json`",
    ]
    return "\n".join(lines) + "\n"


def _render_api_skill_md(
    skill_name: str,
    description: str,
    service: str,
    endpoints: List[Dict[str, Any]],
    auth_type: str,
) -> str:
    lines: List[str] = [
        "---",
        f"name: {skill_name}",
        f"description: {description}",
        "dependencies: []",
        "---",
        "",
        f"# {skill_name}",
        "",
        "This package is generated from `wpscli schema` and is intended for Claude custom skill upload.",
        "",
        "## Scope",
        "",
        f"- Service: `{service}`",
        f"- Endpoint count: `{len(endpoints)}`",
        f"- Default auth template: `{auth_type}`",
        "",
        "## Command Templates",
        "",
    ]
    for ep in endpoints:
        eid = str(ep.get("id", ""))
        summary = str(ep.get("summary") or ep.get("name") or eid)
        lines.append(f"- `{eid}`: {summary}")
        lines.append("")
        lines.append("```bash")
        lines.append(_command_template(service, ep, auth_type))
        lines.append("```")
        lines.append("")
    return "\n".join(lines) + "\n"


def cmd_inspect(args: argparse.Namespace) -> int:
    schema = _load_service_schema(args.wpscli_bin, args.service)
    endpoints = schema.get("endpoints", [])
    selected = _discover(
        [
            {**ep, "service": args.service}
            for ep in endpoints
            if isinstance(ep, dict)
        ],
        query=args.query or "",
        includes=args.include,
        excludes=args.exclude,
        services=[args.service],
        limit=args.limit,
    )
    payload = {
        "mode": "inspect",
        "service": args.service,
        "total_endpoints": len(endpoints),
        "selected_endpoints": len(selected),
        "filters": {
            "query": args.query or "",
            "include": args.include,
            "exclude": args.exclude,
            "limit": args.limit,
        },
        "endpoints": selected,
    }
    text = json.dumps(payload, ensure_ascii=False, indent=2)
    if args.output_file:
        out = Path(args.output_file)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text, encoding="utf-8")
    print(text)
    return 0


def cmd_discover(args: argparse.Namespace) -> int:
    endpoints = _load_all_endpoints()
    explicit_helpers = _uniq(args.helper)
    inferred = _infer_helpers(args.query or "")
    helpers = explicit_helpers or inferred
    services = _uniq(args.service + _helper_services(helpers))
    selected = _discover(
        endpoints,
        query=args.query or "",
        includes=args.include,
        excludes=args.exclude,
        services=services,
        limit=args.limit,
    )
    payload = {
        "mode": "discover",
        "query": args.query or "",
        "helpers": helpers,
        "service_filter": services,
        "total_endpoints": len(endpoints),
        "selected_endpoints": len(selected),
        "items": selected,
    }
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 0


def cmd_create_api(args: argparse.Namespace) -> int:
    _ensure_generated_skills(args.wpscli_bin)
    normalized_name = _normalize_name(args.name)
    description = args.description.strip()
    if not description:
        raise ValueError("description cannot be empty")
    if len(description) > 200:
        raise ValueError("description must be <= 200 characters for Claude skill metadata")

    schema = _load_service_schema(args.wpscli_bin, args.service)
    endpoints = [{**ep, "service": args.service} for ep in schema.get("endpoints", []) if isinstance(ep, dict)]
    selected = _discover(
        endpoints,
        query=args.query or "",
        includes=args.include,
        excludes=args.exclude,
        services=[args.service],
        limit=args.limit,
    )
    if not selected:
        raise RuntimeError("no endpoints selected; adjust query/include/exclude filters")

    output_root = Path(args.output_dir)
    skill_dir = output_root / normalized_name
    if skill_dir.exists() and not args.overwrite:
        raise RuntimeError(f"target exists: {skill_dir} (use --overwrite)")
    skill_dir.mkdir(parents=True, exist_ok=True)

    skill_md = _render_api_skill_md(normalized_name, description, args.service, selected, args.auth_type)
    reference_md = _render_api_reference(selected, args.auth_type)
    commands_json = {
        "skill_name": normalized_name,
        "mode": "api_translation",
        "service": args.service,
        "auth_type": args.auth_type,
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "commands": [
            {
                "service": ep.get("service"),
                "endpoint_id": ep.get("id"),
                "method": ep.get("http_method"),
                "path": ep.get("path"),
                "scopes": ep.get("scopes", []),
                "command": _command_template(str(ep.get("service", "")), ep, args.auth_type),
            }
            for ep in selected
        ],
    }
    manifest = {
        "name": normalized_name,
        "description": description,
        "mode": "api_translation",
        "service": args.service,
        "endpoint_count": len(selected),
        "source": "wpscli schema",
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
    }

    (skill_dir / "Skill.md").write_text(skill_md, encoding="utf-8")
    (skill_dir / "REFERENCE.md").write_text(reference_md, encoding="utf-8")
    (skill_dir / "commands.json").write_text(json.dumps(commands_json, ensure_ascii=False, indent=2), encoding="utf-8")
    (skill_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")

    print(
        json.dumps(
            {
                "ok": True,
                "mode": "api_translation",
                "skill_dir": str(skill_dir),
                "generated_files": ["Skill.md", "REFERENCE.md", "commands.json", "manifest.json"],
                "endpoint_count": len(selected),
            },
            ensure_ascii=False,
            indent=2,
        )
    )
    return 0


def cmd_create_business(args: argparse.Namespace) -> int:
    _ensure_generated_skills(args.wpscli_bin)
    normalized_name = _normalize_name(args.name)
    description = args.description.strip()
    goal = args.goal.strip()
    if not description:
        raise ValueError("description cannot be empty")
    if len(description) > 200:
        raise ValueError("description must be <= 200 characters for Claude skill metadata")
    if not goal:
        raise ValueError("goal cannot be empty")

    explicit_helpers = _uniq(args.helper)
    inferred_helpers = _infer_helpers(goal)
    helpers = explicit_helpers or inferred_helpers or ["files", "doc"]
    unknown = [h for h in helpers if h not in HELPER_PROFILES]
    if unknown:
        raise ValueError(f"unknown helper(s): {', '.join(unknown)}")

    endpoints = _load_all_endpoints()
    service_filter = _uniq(args.service + _helper_services(helpers))
    selected = _discover(
        endpoints,
        query=goal,
        includes=args.include,
        excludes=args.exclude,
        services=service_filter,
        limit=args.limit,
    )
    if not selected:
        selected = _discover(
            endpoints,
            query="",
            includes=args.include,
            excludes=args.exclude,
            services=service_filter,
            limit=args.limit,
        )
    if not selected:
        raise RuntimeError("no endpoints selected for business skill; adjust helper/service/filter settings")

    output_root = Path(args.output_dir)
    skill_dir = output_root / normalized_name
    if skill_dir.exists() and not args.overwrite:
        raise RuntimeError(f"target exists: {skill_dir} (use --overwrite)")
    (skill_dir / "references").mkdir(parents=True, exist_ok=True)
    (skill_dir / "evals").mkdir(parents=True, exist_ok=True)
    (skill_dir / "workspace").mkdir(parents=True, exist_ok=True)

    skill_md = _render_business_skill_md(
        normalized_name, description, goal, helpers, selected, args.auth_type
    )
    helper_md = _render_helper_reference(helpers)
    api_md = _render_api_reference(selected, args.auth_type)
    guardrails_md = _render_workflow_guardrails(normalized_name, goal, helpers)
    evals = {
        "skill_name": normalized_name,
        "evals": _build_eval_prompts(normalized_name, goal, helpers),
    }
    commands_json = {
        "skill_name": normalized_name,
        "mode": "business_orchestration",
        "goal": goal,
        "helpers": helpers,
        "auth_type": args.auth_type,
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "commands": [
            {
                "service": ep.get("service"),
                "endpoint_id": ep.get("id"),
                "method": ep.get("http_method"),
                "path": ep.get("path"),
                "scopes": ep.get("scopes", []),
                "command": _command_template(str(ep.get("service", "")), ep, args.auth_type),
            }
            for ep in selected
        ],
    }
    manifest = {
        "name": normalized_name,
        "description": description,
        "mode": "business_orchestration",
        "goal": goal,
        "helpers": helpers,
        "service_filter": service_filter,
        "endpoint_count": len(selected),
        "source": "descriptors + helper profiles",
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
    }

    (skill_dir / "Skill.md").write_text(skill_md, encoding="utf-8")
    (skill_dir / "references" / "HELPER_COMPOSITION.md").write_text(helper_md, encoding="utf-8")
    (skill_dir / "references" / "API_BASELINE.md").write_text(api_md, encoding="utf-8")
    (skill_dir / "references" / "WORKFLOW_GUARDRAILS.md").write_text(guardrails_md, encoding="utf-8")
    (skill_dir / "evals" / "evals.json").write_text(json.dumps(evals, ensure_ascii=False, indent=2), encoding="utf-8")
    (skill_dir / "commands.json").write_text(json.dumps(commands_json, ensure_ascii=False, indent=2), encoding="utf-8")
    (skill_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    (skill_dir / "workspace" / "README.md").write_text(
        "# Iteration Workspace\n\nUse `iteration-1/`, `iteration-2/` ... to store test outputs and reviews.\n",
        encoding="utf-8",
    )

    print(
        json.dumps(
            {
                "ok": True,
                "mode": "business_orchestration",
                "skill_dir": str(skill_dir),
                "helpers": helpers,
                "service_filter": service_filter,
                "generated_files": [
                    "Skill.md",
                    "references/HELPER_COMPOSITION.md",
                    "references/API_BASELINE.md",
                    "references/WORKFLOW_GUARDRAILS.md",
                    "evals/evals.json",
                    "commands.json",
                    "manifest.json",
                    "workspace/README.md",
                ],
                "endpoint_count": len(selected),
            },
            ensure_ascii=False,
            indent=2,
        )
    )
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description="Create Claude-compatible WPS skills: API translation or business skill orchestration."
    )
    p.add_argument("--wpscli-bin", default="wpscli", help="wpscli executable path")
    sub = p.add_subparsers(dest="command", required=True)

    p_inspect = sub.add_parser("inspect", help="Inspect one service via `wpscli schema`")
    p_inspect.add_argument("--service", required=True, help="wpscli service name")
    p_inspect.add_argument("--query", default="", help="query text")
    p_inspect.add_argument("--include", action="append", default=[], help="endpoint include keyword")
    p_inspect.add_argument("--exclude", action="append", default=[], help="endpoint exclude keyword")
    p_inspect.add_argument("--limit", type=int, default=20, help="max endpoint count (0 for all)")
    p_inspect.add_argument("--output-file", help="optional json output path")
    p_inspect.set_defaults(func=cmd_inspect)

    p_discover = sub.add_parser("discover", help="Discover endpoints across all services")
    p_discover.add_argument("--query", default="", help="business query / intent")
    p_discover.add_argument("--helper", action="append", default=[], help=f"helper hint: {', '.join(sorted(HELPER_PROFILES.keys()))}")
    p_discover.add_argument("--service", action="append", default=[], help="service filter (repeatable)")
    p_discover.add_argument("--include", action="append", default=[], help="endpoint include keyword")
    p_discover.add_argument("--exclude", action="append", default=[], help="endpoint exclude keyword")
    p_discover.add_argument("--limit", type=int, default=40, help="max endpoint count")
    p_discover.set_defaults(func=cmd_discover)

    p_api = sub.add_parser("create-api", help="Legacy mode: create one service API translation skill")
    p_api.add_argument("--name", required=True, help="skill package name")
    p_api.add_argument("--description", required=True, help="skill short description (<=200 chars)")
    p_api.add_argument("--service", required=True, help="wpscli service name")
    p_api.add_argument("--query", default="", help="query text")
    p_api.add_argument("--include", action="append", default=[], help="endpoint include keyword")
    p_api.add_argument("--exclude", action="append", default=[], help="endpoint exclude keyword")
    p_api.add_argument("--limit", type=int, default=20, help="max endpoint count")
    p_api.add_argument("--auth-type", choices=["app", "user"], default="app", help="auth type in templates")
    p_api.add_argument("--output-dir", default="./dist", help="output directory root")
    p_api.add_argument("--overwrite", action="store_true", help="overwrite existing package directory")
    p_api.set_defaults(func=cmd_create_api)

    p_business = sub.add_parser("create-business", help="Create business skill package from helper composition")
    p_business.add_argument("--name", required=True, help="skill package name")
    p_business.add_argument("--description", required=True, help="skill short description (<=200 chars)")
    p_business.add_argument("--goal", required=True, help="business goal statement")
    p_business.add_argument("--helper", action="append", default=[], help=f"helper to compose: {', '.join(sorted(HELPER_PROFILES.keys()))}")
    p_business.add_argument("--service", action="append", default=[], help="extra service filter")
    p_business.add_argument("--include", action="append", default=[], help="endpoint include keyword")
    p_business.add_argument("--exclude", action="append", default=[], help="endpoint exclude keyword")
    p_business.add_argument("--limit", type=int, default=60, help="max endpoint count")
    p_business.add_argument("--auth-type", choices=["app", "user"], default="app", help="auth type in templates")
    p_business.add_argument("--output-dir", default="./dist", help="output directory root")
    p_business.add_argument("--overwrite", action="store_true", help="overwrite existing package directory")
    p_business.set_defaults(func=cmd_create_business)

    # Backward-compatible alias from previous version.
    p_create_alias = sub.add_parser("create", help="Alias of create-business")
    p_create_alias.add_argument("--name", required=True)
    p_create_alias.add_argument("--description", required=True)
    p_create_alias.add_argument("--goal", required=True)
    p_create_alias.add_argument("--helper", action="append", default=[])
    p_create_alias.add_argument("--service", action="append", default=[])
    p_create_alias.add_argument("--include", action="append", default=[])
    p_create_alias.add_argument("--exclude", action="append", default=[])
    p_create_alias.add_argument("--limit", type=int, default=60)
    p_create_alias.add_argument("--auth-type", choices=["app", "user"], default="app")
    p_create_alias.add_argument("--output-dir", default="./dist")
    p_create_alias.add_argument("--overwrite", action="store_true")
    p_create_alias.set_defaults(func=cmd_create_business)

    return p


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return int(args.func(args))
    except Exception as exc:  # pragma: no cover
        print(json.dumps({"ok": False, "error": str(exc)}, ensure_ascii=False, indent=2))
        return 1


if __name__ == "__main__":
    sys.exit(main())
