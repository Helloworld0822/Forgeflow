#!/usr/bin/env bash
# 파일 편집 후 변경분을 자동 커밋한다 (.env 등 민감 파일 제외).
set -euo pipefail

ROOT="$(git -C "${CURSOR_PROJECT_DIR:-.}" rev-parse --show-toplevel 2>/dev/null)" || exit 0
cd "$ROOT"

INPUT="$(cat)"
FILE="$(printf '%s' "$INPUT" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
except json.JSONDecodeError:
    data = {}
print(data.get('file_path') or data.get('path') or '')
" 2>/dev/null || true)"

# 민감·생성 파일은 커밋하지 않음
should_skip() {
  case "$1" in
    .env|.env.*|*/.env|*/.env.*) return 0 ;;
    */node_modules/*|*/target/*|*/.git/*) return 0 ;;
  esac
  return 1
}

if [ -n "$FILE" ] && should_skip "$FILE"; then
  exit 0
fi

LOCK_FILE="$ROOT/.git/autoforge-auto-commit.lock"
exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  exit 0
fi

# 편집된 파일 우선 스테이징, 없으면 추적 중인 변경분 전체
if [ -n "$FILE" ] && [ -e "$FILE" ]; then
  git add -- "$FILE"
else
  git add -u
fi

git reset -q HEAD -- .env .env.local .env.* 2>/dev/null || true

if git diff --cached --quiet; then
  exit 0
fi

REL="${FILE#"$ROOT"/}"
REL="${REL#/}"
if [ -z "$REL" ]; then
  REL="$(git diff --cached --name-only | head -1)"
fi

MSG="edit: ${REL:-project files}"
git commit -m "$MSG"

exit 0
