#!/usr/bin/env bash
# 에이전트 작업 종료(stop) 시 origin에 push한다.
set -euo pipefail

ROOT="$(git -C "${CURSOR_PROJECT_DIR:-.}" rev-parse --show-toplevel 2>/dev/null)" || exit 0
cd "$ROOT"

# upstream 없으면 현재 브랜치 기준으로 설정 시도
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if ! git rev-parse --abbrev-ref '@{u}' >/dev/null 2>&1; then
  git push -u origin "$BRANCH" 2>/dev/null || exit 0
  exit 0
fi

AHEAD="$(git rev-list --count '@{u}..HEAD' 2>/dev/null || echo 0)"
if [ "${AHEAD:-0}" -eq 0 ]; then
  exit 0
fi

git push origin "$BRANCH"
exit 0
