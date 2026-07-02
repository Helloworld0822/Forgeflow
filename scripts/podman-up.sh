#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building AutoForge image with Podman"
podman build -t localhost/autoforge:latest -f Containerfile .

echo "==> Starting stack with podman-compose"
if command -v podman-compose &>/dev/null; then
  podman-compose -f podman-compose.yml up -d --scale worker=3
elif command -v docker-compose &>/dev/null; then
  docker-compose -f podman-compose.yml up -d --scale worker=3
else
  echo "podman-compose not found. Install: pip install podman-compose"
  exit 1
fi

echo ""
echo "AutoForge is running:"
echo "  API:    http://localhost:8080"
echo "  MinIO:  http://localhost:9001 (minioadmin/minioadmin)"
echo "  Redis:  localhost:6379"
echo ""
echo "Scale workers: podman-compose -f podman-compose.yml up -d --scale worker=5"
