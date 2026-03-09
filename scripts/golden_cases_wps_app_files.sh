#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

WPSCLI_BIN="${WPSCLI_BIN:-cargo run --quiet --bin wpscli --}"

run_case() {
  local name="$1"
  local cmd="$2"
  shift 2
  local expected_paths=("$@")

  echo "==> ${name}"
  echo "CMD: ${cmd}"
  local out
  out="$(eval "${WPSCLI_BIN} ${cmd}")"
  python3 - "$out" "${expected_paths[@]}" <<'PY'
import json
import sys

raw = sys.argv[1]
paths = sys.argv[2:]
obj = json.loads(raw)
if obj.get("ok") is not True:
    raise SystemExit(f"expected ok=true, got: {raw}")

def has_path(data, path):
    cur = data
    for part in path.split("."):
        if isinstance(cur, dict) and part in cur:
            cur = cur[part]
        else:
            return False
    return True

for p in paths:
    if not has_path(obj, p):
        raise SystemExit(f"missing json path: {p}")
print("PASS")
PY
}

run_case \
  "list-apps dry-run" \
  "files list-apps --user-token --dry-run" \
  "data.data.scope_preflight.check_mode"

run_case \
  "ensure-app dry-run" \
  "files ensure-app --app Demo/子模块 --user-token --dry-run" \
  "data.data.scope_preflight.check_mode" \
  "data.data.state_paths.base_dir"

run_case \
  "create dry-run workflow" \
  "files create --app Demo/子模块 --file test.otl --user-token --dry-run" \
  "data.data.workflow" \
  "data.data.state_paths.operation_log"

run_case \
  "state view" \
  "files state --limit 5" \
  "data.data.paths.base_dir" \
  "data.data.registry.apps" \
  "data.data.recent_operations"

echo "All dry-run golden cases passed."
