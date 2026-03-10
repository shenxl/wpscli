#!/usr/bin/env python3
"""
Generate reusable Claude skill packages from wpscli schema output.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List


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


def _normalize_name(name: str) -> str:
    value = name.strip().lower()
    value = re.sub(r"[^a-z0-9_-]+", "-", value)
    value = re.sub(r"-{2,}", "-", value).strip("-")
    if not value:
        raise ValueError("skill name is empty after normalization")
    return value


def _text_blob(ep: Dict[str, Any]) -> str:
    fields = [
        str(ep.get("id", "")),
        str(ep.get("name", "")),
        str(ep.get("summary", "")),
        str(ep.get("path", "")),
    ]
    return " ".join(fields).lower()


def _filter_endpoints(
    endpoints: List[Dict[str, Any]],
    includes: List[str],
    excludes: List[str],
    limit: int,
) -> List[Dict[str, Any]]:
    include_terms = [x.lower() for x in includes if x.strip()]
    exclude_terms = [x.lower() for x in excludes if x.strip()]
    out: List[Dict[str, Any]] = []
    for ep in endpoints:
        blob = _text_blob(ep)
        if include_terms and not any(term in blob for term in include_terms):
            continue
        if exclude_terms and any(term in blob for term in exclude_terms):
            continue
        out.append(ep)
    out.sort(key=lambda x: str(x.get("id", "")))
    if limit > 0:
        out = out[:limit]
    return out


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


def _render_skill_md(
    skill_name: str,
    description: str,
    service: str,
    endpoints: List[Dict[str, Any]],
    auth_type: str,
) -> str:
    lines: List[str] = []
    lines.extend(
        [
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
            "## Suggested Workflow",
            "",
            "1. Confirm auth readiness: `wpscli auth status`",
            f"2. Inspect service schema: `wpscli schema {service}`",
            "3. Execute one endpoint at a time and keep JSON outputs for downstream agent chaining.",
            "",
            "## Command Templates",
            "",
        ]
    )
    for ep in endpoints:
        eid = str(ep.get("id", ""))
        summary = str(ep.get("summary") or ep.get("name") or eid)
        lines.append(f"- `{eid}`: {summary}")
        lines.append("")
        lines.append("```bash")
        lines.append(_command_template(service, ep, auth_type))
        lines.append("```")
        lines.append("")
    lines.extend(
        [
            "## References",
            "",
            "- See `REFERENCE.md` for endpoint details and scopes.",
            "- See `commands.json` for machine-readable command metadata.",
            "",
        ]
    )
    return "\n".join(lines)


def _render_reference_md(service: str, endpoints: List[Dict[str, Any]], auth_type: str) -> str:
    lines: List[str] = []
    lines.extend(
        [
            f"# {service} Endpoint Reference",
            "",
            "| id | method | path | scopes | command |",
            "|---|---|---|---|---|",
        ]
    )
    for ep in endpoints:
        eid = str(ep.get("id", ""))
        method = str(ep.get("http_method", "GET")).upper()
        path = str(ep.get("path", ""))
        scopes = ", ".join(ep.get("scopes", []) or [])
        cmd = _command_template(service, ep, auth_type)
        lines.append(f"| `{eid}` | `{method}` | `{path}` | `{scopes or '-'}` | `{cmd}` |")
    lines.append("")
    return "\n".join(lines)


def cmd_inspect(args: argparse.Namespace) -> int:
    schema = _load_service_schema(args.wpscli_bin, args.service)
    endpoints = schema.get("endpoints", [])
    selected = _filter_endpoints(endpoints, args.include, args.exclude, args.limit)
    payload = {
        "service": args.service,
        "total_endpoints": len(endpoints),
        "selected_endpoints": len(selected),
        "filters": {
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


def cmd_create(args: argparse.Namespace) -> int:
    normalized_name = _normalize_name(args.name)
    description = args.description.strip()
    if not description:
        raise ValueError("description cannot be empty")
    if len(description) > 200:
        raise ValueError("description must be <= 200 characters for Claude skill metadata")

    schema = _load_service_schema(args.wpscli_bin, args.service)
    endpoints = schema.get("endpoints", [])
    selected = _filter_endpoints(endpoints, args.include, args.exclude, args.limit)
    if not selected:
        raise RuntimeError("no endpoints selected; adjust include/exclude filters")

    output_root = Path(args.output_dir)
    skill_dir = output_root / normalized_name
    if skill_dir.exists() and not args.overwrite:
        raise RuntimeError(f"target exists: {skill_dir} (use --overwrite)")
    skill_dir.mkdir(parents=True, exist_ok=True)

    skill_md = _render_skill_md(normalized_name, description, args.service, selected, args.auth_type)
    reference_md = _render_reference_md(args.service, selected, args.auth_type)
    commands_json = {
        "skill_name": normalized_name,
        "service": args.service,
        "auth_type": args.auth_type,
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "commands": [
            {
                "endpoint_id": ep.get("id"),
                "method": ep.get("http_method"),
                "path": ep.get("path"),
                "scopes": ep.get("scopes", []),
                "command": _command_template(args.service, ep, args.auth_type),
            }
            for ep in selected
        ],
    }
    manifest = {
        "name": normalized_name,
        "description": description,
        "service": args.service,
        "endpoint_count": len(selected),
        "source": "wpscli schema",
        "wpscli_bin": args.wpscli_bin,
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
    }

    (skill_dir / "Skill.md").write_text(skill_md, encoding="utf-8")
    (skill_dir / "REFERENCE.md").write_text(reference_md, encoding="utf-8")
    (skill_dir / "commands.json").write_text(
        json.dumps(commands_json, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    (skill_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    print(
        json.dumps(
            {
                "ok": True,
                "skill_dir": str(skill_dir),
                "generated_files": ["Skill.md", "REFERENCE.md", "commands.json", "manifest.json"],
                "endpoint_count": len(selected),
            },
            ensure_ascii=False,
            indent=2,
        )
    )
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Generate Claude-style WPS skills from wpscli schema.")
    p.add_argument("--wpscli-bin", default="wpscli", help="wpscli executable path")
    sub = p.add_subparsers(dest="command", required=True)

    p_inspect = sub.add_parser("inspect", help="Inspect service endpoints via wpscli schema")
    p_inspect.add_argument("--service", required=True, help="wpscli service name")
    p_inspect.add_argument("--include", action="append", default=[], help="endpoint include keyword")
    p_inspect.add_argument("--exclude", action="append", default=[], help="endpoint exclude keyword")
    p_inspect.add_argument("--limit", type=int, default=20, help="max endpoint count (0 for all)")
    p_inspect.add_argument("--output-file", help="optional json file output path")
    p_inspect.set_defaults(func=cmd_inspect)

    p_create = sub.add_parser("create", help="Create reusable skill package")
    p_create.add_argument("--name", required=True, help="skill package name")
    p_create.add_argument("--description", required=True, help="skill short description (<=200 chars)")
    p_create.add_argument("--service", required=True, help="wpscli service name")
    p_create.add_argument("--include", action="append", default=[], help="endpoint include keyword")
    p_create.add_argument("--exclude", action="append", default=[], help="endpoint exclude keyword")
    p_create.add_argument("--limit", type=int, default=20, help="max endpoint count (0 for all)")
    p_create.add_argument("--auth-type", choices=["app", "user"], default="app", help="auth type in command templates")
    p_create.add_argument("--output-dir", default="./dist", help="output directory root")
    p_create.add_argument("--overwrite", action="store_true", help="overwrite existing package directory")
    p_create.set_defaults(func=cmd_create)

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
