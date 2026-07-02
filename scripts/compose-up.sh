#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
WORKER_SCALE="${WORKER_SCALE:-3}"

echo "==> Building and starting AutoForge stack"
if command -v docker &>/dev/null && docker compose version &>/dev/null 2>&1; then
  docker compose -f "$COMPOSE_FILE" up -d --build --scale "worker=${WORKER_SCALE}"
elif command -v podman-compose &>/dev/null; then
  podman-compose -f podman-compose.yml up -d --build --scale "worker=${WORKER_SCALE}"
else
  echo "docker compose or podman-compose required"
  exit 1
fi

echo ""
echo "AutoForge is running:"
echo "  Web (nginx):  http://localhost"
echo "  API (proxy):  http://localhost/v1"
echo "  Images:       http://localhost/media/{filename}"
echo ""
echo "Scale workers: WORKER_SCALE=5 ./scripts/compose-up.sh"
