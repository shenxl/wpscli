#!/usr/bin/env python3
"""
Generate command docs for wpscli from live `--help` outputs.

Usage:
  python3 scripts/generate_docs.py
  python3 scripts/generate_docs.py --check
"""

from __future__ import annotations

import argparse
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Dict, List


REPO_ROOT = Path(__file__).resolve().parents[1]


DOC_SPECS: List[Dict] = [
    {
        "file": "docs/commands/01-root.md",
        "title": "顶层命令",
        "desc": "`wpscli` 顶层入口与全局参数。",
        "entries": [
            {"label": "根命令帮助", "cmd": "--help"},
        ],
    },
    {
        "file": "docs/commands/02-auth.md",
        "title": "认证命令",
        "desc": "`auth` 相关子命令，覆盖 setup/login/refresh/status/harden/logout。",
        "entries": [
            {"label": "auth", "cmd": "auth --help"},
            {"label": "auth setup", "cmd": "auth setup --help"},
            {"label": "auth login", "cmd": "auth login --help"},
            {"label": "auth harden", "cmd": "auth harden --help"},
            {"label": "auth status", "cmd": "auth status --help"},
        ],
    },
    {
        "file": "docs/commands/03-doc.md",
        "title": "文档助手命令",
        "desc": "`doc` 文档读写与链接解析能力。",
        "entries": [
            {"label": "doc", "cmd": "doc --help"},
            {"label": "doc read-doc", "cmd": "doc read-doc --help"},
            {"label": "doc write-doc", "cmd": "doc write-doc --help"},
        ],
    },
    {
        "file": "docs/commands/04-files-users.md",
        "title": "应用文件与用户助手命令",
        "desc": "`files` 和 `users` 业务助手命令。",
        "entries": [
            {"label": "files", "cmd": "files --help"},
            {"label": "users", "cmd": "users --help"},
        ],
    },
    {
        "file": "docs/commands/05-dbsheet.md",
        "title": "多维表命令",
        "desc": "`dbsheet` SQL-like 命令与 `dbt` 兼容命令。",
        "entries": [
            {"label": "dbsheet", "cmd": "dbsheet --help"},
            {"label": "dbt", "cmd": "dbt --help"},
        ],
    },
    {
        "file": "docs/commands/06-integrations.md",
        "title": "其他业务助手命令",
        "desc": "`calendar/chat/meeting/airpage` 助手命令。",
        "entries": [
            {"label": "calendar", "cmd": "calendar --help"},
            {"label": "chat", "cmd": "chat --help"},
            {"label": "meeting", "cmd": "meeting --help"},
            {"label": "airpage", "cmd": "airpage --help"},
        ],
    },
    {
        "file": "docs/commands/07-system.md",
        "title": "系统命令",
        "desc": "框架级命令：schema/catalog/raw/generate-skills/completions/ui/doctor。",
        "entries": [
            {"label": "schema", "cmd": "schema --help"},
            {"label": "catalog", "cmd": "catalog --help"},
            {"label": "raw", "cmd": "raw --help"},
            {"label": "generate-skills", "cmd": "generate-skills --help"},
            {"label": "completions", "cmd": "completions --help"},
            {"label": "ui", "cmd": "ui --help"},
            {"label": "doctor", "cmd": "doctor --help"},
        ],
    },
]


def run_help(cmd: str) -> str:
    args = shlex.split(cmd)
    full_cmd = ["cargo", "run", "--quiet", "--bin", "wpscli", "--"] + args
    proc = subprocess.run(
        full_cmd,
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(full_cmd)}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    return proc.stdout.strip()


def render_page(spec: Dict) -> str:
    lines: List[str] = []
    lines.append(f"# {spec['title']}")
    lines.append("")
    lines.append("> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。")
    lines.append("")
    lines.append(spec["desc"])
    lines.append("")
    lines.append("---")
    lines.append("")
    for entry in spec["entries"]:
        label = entry["label"]
        cmd = entry["cmd"]
        help_out = run_help(cmd)
        display_cmd = f"wpscli {cmd}".strip()
        lines.append(f"## {label}")
        lines.append("")
        lines.append("```bash")
        lines.append(display_cmd)
        lines.append("```")
        lines.append("")
        lines.append("```text")
        lines.append(help_out)
        lines.append("```")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def render_index() -> str:
    lines: List[str] = []
    lines.append("# 命令文档索引")
    lines.append("")
    lines.append("> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。")
    lines.append("")
    lines.append("- [顶层命令](01-root.md)")
    lines.append("- [认证命令](02-auth.md)")
    lines.append("- [文档助手命令](03-doc.md)")
    lines.append("- [应用文件与用户助手命令](04-files-users.md)")
    lines.append("- [多维表命令](05-dbsheet.md)")
    lines.append("- [其他业务助手命令](06-integrations.md)")
    lines.append("- [系统命令](07-system.md)")
    lines.append("")
    return "\n".join(lines)


def upsert(path: Path, content: str, check: bool) -> bool:
    old = path.read_text(encoding="utf-8") if path.exists() else ""
    changed = old != content
    if check:
        return changed
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return changed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="Only check whether docs are up-to-date.")
    args = parser.parse_args()

    changed_files: List[str] = []
    for spec in DOC_SPECS:
        out = render_page(spec)
        target = REPO_ROOT / spec["file"]
        if upsert(target, out, args.check):
            changed_files.append(str(target.relative_to(REPO_ROOT)))

    index_target = REPO_ROOT / "docs/commands/README.md"
    if upsert(index_target, render_index(), args.check):
        changed_files.append(str(index_target.relative_to(REPO_ROOT)))

    if args.check and changed_files:
        print("Command docs are out of date. Regenerate with:")
        print("  python3 scripts/generate_docs.py")
        print("")
        for f in changed_files:
            print(f"- {f}")
        return 1

    if not args.check:
        print("Docs generated:")
        for spec in DOC_SPECS:
            print(f"- {spec['file']}")
        print("- docs/commands/README.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
