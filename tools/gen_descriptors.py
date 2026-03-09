#!/usr/bin/env python3
"""
Generate WPS service descriptors from wps-openapi-nav data.

Input:
  - wpsskill/wps-openapi-nav/data/api_index.json
  - wpsskill/wps-openapi-nav/data/raw/**/*.json

Output:
  - wps-cli/descriptors/<service>.json
  - wps-cli/descriptors/index.json
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Dict, List, Optional, Tuple


HTTP_METHODS = {"GET", "POST", "PUT", "PATCH", "DELETE"}


@dataclass
class Param:
    name: str
    location: str
    ptype: str
    required: bool
    description: str = ""
    enum: Optional[List[str]] = None


def normalize_endpoint_path(raw: str) -> str:
    if not raw:
        return ""
    path = raw
    if path.startswith("https://openapi.wps.cn"):
        path = path[len("https://openapi.wps.cn") :]
    path = path.strip()
    path = re.sub(r"\*+$", "", path)
    return path


def service_from_path(path: str) -> str:
    if not path:
        return "misc"
    if path.startswith("/oauth2/"):
        return "oauth2"
    if not path.startswith("/v7/"):
        return "misc"
    rest = path[len("/v7/") :]
    if not rest:
        return "misc"
    first = rest.split("/", 1)[0]
    if first == "coop":
        parts = rest.split("/")
        if len(parts) >= 2:
            return f"coop_{parts[1]}"
        return "coop"
    return first


def parse_table_params(section: str, location: str) -> List[Param]:
    lines = [ln for ln in section.splitlines() if "|" in ln]
    if len(lines) < 2:
        return []

    header = [c.strip().lower() for c in lines[0].strip().strip("|").split("|")]

    def col_idx(candidates: List[str], default: int) -> int:
        for i, h in enumerate(header):
            if any(c in h for c in candidates):
                return i
        return default

    name_i = col_idx(["属性名", "名称", "name", "header"], 0)
    type_i = col_idx(["类型", "type"], 1)
    req_i = col_idx(["必填", "required"], 2)
    desc_i = col_idx(["说明", "描述", "description"], 3)
    enum_i = col_idx(["可选值", "enum"], -1)

    out: List[Param] = []
    header_passed = False
    for ln in lines[1:]:
        content = ln.replace("|", "").strip()
        if re.fullmatch(r"[-:\s]+", content):
            header_passed = True
            continue
        if not header_passed:
            continue
        cells = [c.strip() for c in ln.strip().strip("|").split("|")]
        if len(cells) < 2:
            continue
        name = re.sub(r"<[^>]+>", "", cells[name_i] if name_i < len(cells) else "").strip()
        name = name.replace("`", "")
        if not name or name.lower() in {
            "name",
            "type",
            "required",
            "description",
            "属性名",
            "参数",
            "参数类型",
        }:
            continue
        ptype = (cells[type_i] if type_i < len(cells) else "string").replace("`", "").strip()
        required_raw = (cells[req_i] if req_i < len(cells) else "").strip().lower()
        required = required_raw in {"是", "yes", "true", "required"}
        desc = re.sub(r"<[^>]+>", "", cells[desc_i] if desc_i < len(cells) else "").strip()
        enum_vals = None
        if 0 <= enum_i < len(cells):
            enum_txt = re.sub(r"<[^>]+>", "", cells[enum_i]).strip()
            if enum_txt and enum_txt != "-":
                enum_vals = [x.strip() for x in re.split(r"[,/，\s]+", enum_txt) if x.strip()]
        out.append(
            Param(
                name=name,
                location=location,
                ptype=ptype or "string",
                required=required,
                description=desc,
                enum=enum_vals,
            )
        )
    return out


def extract_section(content: str, patterns: List[str]) -> Optional[str]:
    for pat in patterns:
        m = re.search(pat, content, flags=re.IGNORECASE | re.MULTILINE | re.DOTALL)
        if m:
            return m.group(0)
    return None


def extract_doc_params(content: str) -> Tuple[List[Param], List[Param], List[Param]]:
    path_sec = extract_section(
        content,
        [
            r"##\s*路径参数[\s\S]*?(?=\n##|\Z)",
            r"##\s*Path\s*参数[\s\S]*?(?=\n##|\Z)",
        ],
    )
    query_sec = extract_section(
        content,
        [
            r"##\s*查询参数[\s\S]*?(?=\n##|\Z)",
            r"##\s*Query\s*参数[\s\S]*?(?=\n##|\Z)",
        ],
    )
    header_sec = extract_section(
        content,
        [
            r"##\s*请求头[\s\S]*?(?=\n##|\Z)",
            r"##\s*Headers?[\s\S]*?(?=\n##|\Z)",
        ],
    )
    path_params = parse_table_params(path_sec or "", "path")
    query_params = parse_table_params(query_sec or "", "query")
    header_params = parse_table_params(header_sec or "", "header")
    return path_params, query_params, header_params


def load_raw_doc_index(raw_root: Path) -> Dict[int, dict]:
    index: Dict[int, dict] = {}
    for fp in raw_root.rglob("*.json"):
        try:
            payload = json.loads(fp.read_text(encoding="utf-8"))
        except Exception:
            continue
        data = payload.get("data", {})
        meta = data.get("meta", {})
        doc_id = None
        if fp.stem.isdigit():
            doc_id = int(fp.stem)
        else:
            meta_id = str(meta.get("doc_id", "")).strip()
            if meta_id.isdigit():
                doc_id = int(meta_id)
            else:
                version_desc = str(meta.get("version", {}).get("description", ""))
                m = re.search(r"document\s+(\d+)", version_desc, flags=re.IGNORECASE)
                if m:
                    doc_id = int(m.group(1))
        if doc_id is not None:
            index[doc_id] = data
    return index


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate WPS descriptors from openapi data")
    parser.add_argument("--root", required=True, help="Repository root (googlews)")
    parser.add_argument("--output", required=True, help="Descriptor output directory")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)

    api_index_file = root / "wpsskill" / "wps-openapi-nav" / "data" / "api_index.json"
    raw_root = root / "wpsskill" / "wps-openapi-nav" / "data" / "raw"

    api_index = json.loads(api_index_file.read_text(encoding="utf-8"))
    endpoints = api_index.get("endpoints", {})
    raw_docs = load_raw_doc_index(raw_root)

    grouped: Dict[str, dict] = {}
    for endpoint_id, ep in endpoints.items():
        method = str(ep.get("method", "GET")).upper()
        if method not in HTTP_METHODS:
            method = "GET"
        full_path = normalize_endpoint_path(str(ep.get("path", "")))
        if not full_path:
            continue
        service = service_from_path(full_path)
        group = grouped.setdefault(
            service, {"service": service, "base_url": "https://openapi.wps.cn", "endpoints": []}
        )

        doc_id = ep.get("doc_id")
        raw = raw_docs.get(int(doc_id)) if isinstance(doc_id, int) or str(doc_id).isdigit() else None
        content = raw.get("content", "") if raw else ""
        path_params, query_params, header_params = extract_doc_params(content)
        path_placeholders = sorted(set(re.findall(r"\{([^{}]+)\}", full_path)))
        existed = {p.name for p in path_params}
        for p in path_placeholders:
            if p not in existed:
                path_params.append(Param(name=p, location="path", ptype="string", required=True))

        scopes = []
        perm = str(ep.get("permission", "")).strip()
        if perm and perm != "-":
            scopes = [x.strip() for x in perm.split(",") if x.strip()]

        endpoint = {
            "id": endpoint_id,
            "doc_id": doc_id,
            "name": ep.get("name", endpoint_id),
            "summary": ep.get("summary", ""),
            "http_method": method,
            "path": full_path,
            "signature": ep.get("signature", "KSO-1"),
            "scopes": scopes,
            "params": {
                "path": [asdict(p) for p in path_params],
                "query": [asdict(p) for p in query_params],
                "header": [asdict(p) for p in header_params],
            },
        }
        group["endpoints"].append(endpoint)

    service_files = []
    for service in sorted(grouped):
        group = grouped[service]
        group["endpoints"].sort(key=lambda x: x["id"])
        out_file = output / f"{service}.json"
        out_file.write_text(json.dumps(group, ensure_ascii=False, indent=2), encoding="utf-8")
        service_files.append({"service": service, "file": out_file.name, "count": len(group["endpoints"])})

    manifest = {
        "version": "1.0.0",
        "generated_from": str(api_index_file),
        "total_services": len(service_files),
        "total_endpoints": sum(s["count"] for s in service_files),
        "services": service_files,
    }
    (output / "index.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(
        json.dumps(
            {"ok": True, "services": manifest["total_services"], "endpoints": manifest["total_endpoints"]},
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()

