# Podman 가상화 배포 가이드

AutoForge를 **Podman** 컨테이너로 분산 실행합니다. API, Orchestrator, Worker가 분리되어 **Redis Streams** 메시지 큐로 통신합니다.

## 아키텍처

```
  브라우저 ──► nginx:80 ──┬── / (React SPA)
                          ├── /v1/* ──► api:8080
                          └── /health ──► api:8080

  api / orchestrator / worker ──► Redis Streams MQ
              │
      공유 볼륨 artifacts-data (파이프라인 산출물 + 이미지 호스팅)
```

| 서비스 | 역할 |
|--------|------|
| **nginx** | 프론트엔드 정적 파일 + API 리버스 프록시 |
| **api** | Rust REST API (`backend/`) |
| **orchestrator** | 이벤트 소비 → 다음 스테이지 enqueue |
| **worker** | 커맨드 소비 → 스테이지 실행 (수평 확장) |
| **redis** | MQ + 프로젝트 상태 |

아티팩트 저장소는 별도 서비스 없이 `artifacts-data` 공유 볼륨(로컬 디스크)을 사용합니다.
api/orchestrator/worker 컨테이너가 모두 같은 볼륨을 `/data/artifacts`에 마운트하므로
스테이지 산출물과 업로드 이미지가 프로세스 간에 공유됩니다.

## 메시지 큐 최적화

| 스트림 | 용도 | Consumer |
|--------|------|----------|
| `autoforge:commands` | 스테이지 실행 커맨드 | Worker (수평 확장) |
| `autoforge:events` | 완료/실패 이벤트 | Orchestrator |

**최적화 포인트:**
- Worker 3+ replica로 병렬 스테이지 처리 (`architect` ∥ `design`)
- Redis Consumer Group으로 at-least-once + 자동 재분배
- API는 커맨드 enqueue만 — 블로킹 없음
- 프로젝트 상태는 Redis 문자열 키(`autoforge:project:{id}`)에 JSON으로 영속화 (컨테이너 재시작 안전)

## 빠른 시작

```bash
cp .env.example .env
# .env 편집

chmod +x scripts/compose-up.sh scripts/podman-up.sh
./scripts/compose-up.sh
# Podman: ./scripts/podman-up.sh
```

또는 수동:

```bash
docker compose up -d --build --scale worker=3
# Podman: podman-compose -f podman-compose.yml up -d --build --scale worker=3
```

접속: **http://localhost** (nginx)

## 프로세스 역할

| 컨테이너 | 명령 | 역할 |
|----------|------|------|
| `nginx` | — | React SPA + API 프록시 (:80) |
| `api` | `serve` | REST API (`backend/`) |
| `orchestrator` | `orchestrate` | 이벤트 소비 → 다음 스테이지 enqueue |
| `worker` | `worker` | 커맨드 소비 → 스테이지 실행 |

## Worker 스케일링

```bash
# worker 5개로 확장
podman-compose -f podman-compose.yml up -d --scale worker=5

# 특정 스테이지만 처리
podman run --rm -e STAGE_FILTER=implement localhost/autoforge-api:latest worker
```

## Slack 진행률 알림

Slack Incoming Webhook 또는 Bot Token으로 파이프라인 진행률을 실시간 확인합니다.

```bash
# Webhook (간단)
export SLACK_WEBHOOK_URL=https://hooks.slack.com/services/T.../B.../xxx

# Bot Token (스레드 업데이트 지원)
export SLACK_BOT_TOKEN=xoxb-...
export SLACK_CHANNEL=#autoforge
```

**알림 예시:**
```
🚀 AutoForge — 프로젝트 `쇼핑몰 리뉴얼` 시작
✅ ingest  ✅ summarize  🔄 architect  ⏳ design ...
진행률: 45%

🔄 verify — running
진행률: 78% | 상태: running

🎉 파이프라인 완료 — `쇼핑몰 리뉴얼`
진행률: 100%
```

## 환경 변수

| 변수 | 기본값 | 설명 |
|------|--------|------|
| `MESSAGE_QUEUE_ENABLED` | `true` (Podman) | MQ 모드 활성화 |
| `REDIS_URL` | `redis://redis:6379` | Redis 연결 |
| `QUEUE_COMMANDS_STREAM` | `autoforge:commands` | 커맨드 스트림 |
| `QUEUE_EVENTS_STREAM` | `autoforge:events` | 이벤트 스트림 |
| `WORKER_CONCURRENCY` | `4` | Worker 동시 처리 수 |
| `SLACK_WEBHOOK_URL` | — | Slack Webhook |

## 로컬 개발 (MQ 없이)

```bash
MESSAGE_QUEUE_ENABLED=false cargo run
```

단일 프로세스 인라인 모드로 실행됩니다.
