#!/usr/bin/env python3
"""
WPS 故障看板日报脚本

用途：
1) 聚合最近 N 小时的 API 失败与技能事件
2) 输出 JSON 或 Markdown 报告
3) 适配 cron 定时巡检
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

from token_manager import get_token_status
from wps_platform import PlatformConfig, build_platform_health_snapshot


OAUTH_DIR = Path(__file__).parent
REPORT_DIR = OAUTH_DIR / "data" / "reports"


SUGGESTION_BY_ERROR = {
    "auth": "检查用户 Token 是否过期，必要时重新授权。",
    "scope": "检查缺失 scope 并通过 OAuth 扩权后重试。",
    "rate_limit": "降低调用频率，增加退避重试或分批执行。",
    "timeout": "提高超时时间，检查网络链路与上游响应速度。",
    "network": "排查本机网络/代理配置，确认可访问 openapi.wps.cn。",
    "upstream": "上游服务不稳定，建议稍后重试并保留 request_id。",
    "validation": "检查参数格式、必填字段和 ID 是否正确。",
    "not_found": "确认资源 ID 是否有效，避免使用已删除对象。",
}


def _to_server_token_status() -> dict:
    raw = get_token_status()
    return {
        "user": {
            "valid": raw.get("user_token_valid", False),
            "ttl_human": f"{raw.get('user_token_ttl', 0)}s",
            "scope_list": raw.get("user_scope_list", []),
            "has_refresh_token": raw.get("has_refresh_token", False),
        },
        "app": {
            "valid": raw.get("app_token_valid", False),
            "ttl_human": f"{raw.get('app_token_ttl', 0)}s",
        },
    }


def _build_markdown(snapshot: dict) -> str:
    api = snapshot.get("api", {})
    token = snapshot.get("token", {})
    now = datetime.now(timezone.utc).astimezone().strftime("%Y-%m-%d %H:%M:%S %Z")
    lines = [
        "# WPS 平台故障日报",
        "",
        f"- 生成时间: {now}",
        f"- 健康状态: **{snapshot.get('overall', 'unknown')}**",
        f"- 观察窗口: 最近 {snapshot.get('lookback_hours', 24)} 小时",
        "",
        "## Token 状态",
        "",
        f"- 用户 Token: {'OK' if token.get('user_ok') else 'FAIL'}",
        f"- 应用 Token: {'OK' if token.get('app_ok') else 'FAIL'}",
        f"- Refresh Token: {'OK' if token.get('has_refresh_token') else 'FAIL'}",
        f"- Scope 数量: {token.get('scope_count', 0)}",
        "",
        "## API 失败概览",
        "",
        f"- 总请求数: {api.get('total', 0)}",
        f"- 失败数: {api.get('failed', 0)}",
        f"- 失败率: {api.get('failure_rate', 0)}%",
        "",
        "## 失败类型 Top",
        "",
    ]

    error_types = api.get("top_error_types", [])
    if not error_types:
        lines.append("- 无失败记录")
    else:
        for item in error_types:
            error_type = item.get("error_type", "unknown")
            count = item.get("count", 0)
            suggestion = SUGGESTION_BY_ERROR.get(error_type, "请结合 request_id 与日志排查。")
            lines.append(f"- `{error_type}`: {count} 次；建议：{suggestion}")

    lines.extend(["", "## 失败接口 Top", ""])
    top_paths = api.get("top_paths", [])
    if not top_paths:
        lines.append("- 无失败接口")
    else:
        for item in top_paths:
            lines.append(f"- `{item.get('path', 'unknown')}`: {item.get('count', 0)} 次")

    latest = api.get("latest_failure") or {}
    if latest:
        lines.extend([
            "",
            "## 最近一次失败",
            "",
            f"- 时间: {latest.get('time', '-')}",
            f"- 方法: {latest.get('method', '-')}",
            f"- 路径: {latest.get('path', '-')}",
            f"- HTTP: {latest.get('http_status', '-')}",
            f"- 类型: {latest.get('error_type', '-')}",
            f"- request_id: {latest.get('request_id', '-')}",
        ])

    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description="WPS 故障看板日报生成")
    parser.add_argument("--hours", type=int, default=24, help="统计窗口（小时）")
    parser.add_argument("--format", choices=["json", "markdown"], default="markdown", help="输出格式")
    parser.add_argument("--output", help="输出文件路径（默认打印到 stdout）")
    args = parser.parse_args()

    config = PlatformConfig.from_env(OAUTH_DIR)
    snapshot = build_platform_health_snapshot(
        config=config,
        token_status=_to_server_token_status(),
        lookback_hours=max(1, args.hours),
    )

    if args.format == "json":
        payload = json.dumps(snapshot, ensure_ascii=False, indent=2)
    else:
        payload = _build_markdown(snapshot)

    if args.output:
        out_path = Path(args.output)
    else:
        REPORT_DIR.mkdir(parents=True, exist_ok=True)
        suffix = "json" if args.format == "json" else "md"
        ts = datetime.now().strftime("%Y%m%d-%H%M%S")
        out_path = REPORT_DIR / f"wps-fault-report-{ts}.{suffix}"

    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(payload)

    print(f"✅ 报告已生成: {out_path}")
    if not args.output:
        print(payload if args.format == "markdown" else json.dumps(snapshot, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
