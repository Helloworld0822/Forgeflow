#!/usr/bin/env bash
# Podman 호환 래퍼 — compose-up.sh 사용 권장
set -euo pipefail
export COMPOSE_FILE=podman-compose.yml
exec "$(dirname "$0")/compose-up.sh" "$@"
