#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
HOOK_DIR="$ROOT_DIR/.githooks"

mkdir -p "$HOOK_DIR"

cat > "$HOOK_DIR/pre-commit" <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
python3 scripts/generate_docs.py --check
HOOK

chmod +x "$HOOK_DIR/pre-commit"
git config core.hooksPath .githooks

echo "Installed git hooks at .githooks/"
echo "pre-commit will now enforce docs sync via scripts/generate_docs.py --check"
