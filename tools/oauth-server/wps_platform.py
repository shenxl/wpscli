#!/usr/bin/env python3
"""
WPS Skill Platform

统一平台层：为所有 wps-* 技能提供
1) 结构化事件日志
2) API 错误聚合
3) 健康快照
"""

from __future__ import annotations

import json
import os
from collections import Counter
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


def _parse_iso_datetime(value: str) -> datetime | None:
    if not value:
        return None
    try:
        normalized = value.replace("Z", "+00:00")
        dt = datetime.fromisoformat(normalized)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt.astimezone(timezone.utc)
    except Exception:
        return None


@dataclass
class PlatformConfig:
    oauth_dir: Path
    api_events_path: Path
    skill_events_path: Path
    max_tail_lines: int = 3000

    @classmethod
    def from_env(cls, oauth_dir: Path) -> "PlatformConfig":
        api_events = Path(
            os.environ.get(
                "WPS_API_LOG_PATH",
                str(oauth_dir / "data" / "wps_api_events.jsonl"),
            )
        )
        skill_events = Path(
            os.environ.get(
                "WPS_SKILL_EVENT_LOG_PATH",
                str(oauth_dir / "data" / "wps_skill_events.jsonl"),
            )
        )
        max_tail = int(os.environ.get("WPS_DASHBOARD_MAX_TAIL_LINES", "3000"))
        return cls(
            oauth_dir=oauth_dir,
            api_events_path=api_events,
            skill_events_path=skill_events,
            max_tail_lines=max(200, max_tail),
        )


def _read_jsonl_tail(path: Path, max_lines: int) -> List[Dict[str, Any]]:
    if not path.exists():
        return []

    lines: List[str] = []
    try:
        with open(path, "r", encoding="utf-8") as f:
            lines = f.readlines()
    except Exception:
        return []

    output: List[Dict[str, Any]] = []
    for line in lines[-max_lines:]:
        line = line.strip()
        if not line:
            continue
        try:
            payload = json.loads(line)
            if isinstance(payload, dict):
                output.append(payload)
        except Exception:
            continue
    return output


def write_skill_event(config: PlatformConfig, event: Dict[str, Any]) -> None:
    record = {
        "time": _utc_now().isoformat().replace("+00:00", "Z"),
        **event,
    }
    try:
        config.skill_events_path.parent.mkdir(parents=True, exist_ok=True)
        with open(config.skill_events_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")
    except Exception:
        # 观测失败不能影响业务流程
        return


def _within_hours(rows: Iterable[Dict[str, Any]], hours: int) -> List[Dict[str, Any]]:
    deadline = _utc_now() - timedelta(hours=max(1, hours))
    output: List[Dict[str, Any]] = []
    for row in rows:
        ts = _parse_iso_datetime(str(row.get("time", "")))
        if ts and ts >= deadline:
            output.append(row)
    return output


def aggregate_api_failures(rows: Iterable[Dict[str, Any]]) -> Dict[str, Any]:
    row_list = list(rows)
    failures: List[Dict[str, Any]] = []
    for row in row_list:
        http_status = int(row.get("http_status", 0) or 0)
        code = int(row.get("code", 0) or 0)
        if http_status >= 400 or code != 0:
            failures.append(row)

    by_error_type = Counter((r.get("error_type") or "unknown") for r in failures)
    by_path = Counter((r.get("path") or "unknown") for r in failures)
    by_status = Counter(str(r.get("http_status", "0")) for r in failures)

    top_paths = [{"path": p, "count": c} for p, c in by_path.most_common(8)]
    top_error_types = [{"error_type": t, "count": c} for t, c in by_error_type.most_common(8)]
    top_status = [{"http_status": s, "count": c} for s, c in by_status.most_common(8)]

    latest_failure = failures[-1] if failures else None
    return {
        "total": len(row_list),
        "failed": len(failures),
        "failure_rate": round((len(failures) / len(row_list) * 100), 2) if row_list else 0.0,
        "top_paths": top_paths,
        "top_error_types": top_error_types,
        "top_http_status": top_status,
        "latest_failure": latest_failure,
    }


def build_platform_health_snapshot(
    config: PlatformConfig,
    token_status: Dict[str, Any],
    lookback_hours: int = 24,
) -> Dict[str, Any]:
    api_rows = _read_jsonl_tail(config.api_events_path, config.max_tail_lines)
    skill_rows = _read_jsonl_tail(config.skill_events_path, config.max_tail_lines)
    api_recent = _within_hours(api_rows, lookback_hours)
    skill_recent = _within_hours(skill_rows, lookback_hours)

    agg = aggregate_api_failures(api_recent)

    user_ok = bool(token_status.get("user", {}).get("valid"))
    app_ok = bool(token_status.get("app", {}).get("valid"))
    has_refresh = bool(token_status.get("user", {}).get("has_refresh_token"))

    if user_ok and app_ok and has_refresh and agg["failure_rate"] < 20:
        overall = "healthy"
    elif user_ok or app_ok:
        overall = "degraded"
    else:
        overall = "unhealthy"

    return {
        "time": _utc_now().isoformat().replace("+00:00", "Z"),
        "overall": overall,
        "lookback_hours": lookback_hours,
        "token": {
            "user_ok": user_ok,
            "app_ok": app_ok,
            "has_refresh_token": has_refresh,
            "user_ttl": token_status.get("user", {}).get("ttl_human"),
            "app_ttl": token_status.get("app", {}).get("ttl_human"),
            "scope_count": len(token_status.get("user", {}).get("scope_list", [])),
        },
        "api": agg,
        "skill_events": {
            "count": len(skill_recent),
            "latest": skill_recent[-20:],
        },
        "paths": {
            "api_events": str(config.api_events_path),
            "skill_events": str(config.skill_events_path),
        },
    }

