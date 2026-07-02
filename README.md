# AutoForge — AI 외주 자동화 프로그램

PDF 계획서를 업로드하면 **요약(Sonnet) → 기획(Fable) → 디자인(Stitch) → 구현(Codex 5.3)** 파이프라인이 자동 실행되는 **단일 실행형 Rust 프로그램**입니다.

웹 UI는 **Actix-web** 기반으로 제공됩니다.

## 모델 역할

| 단계 | 모델 | 역할 |
|------|------|------|
| Summarize | `claude-4.6-sonnet-high-thinking` | PDF 계획서 구조화 요약 |
| Architect | `claude-fable-5-thinking-high` | 시스템 아키텍처 + 상세 기획 |
| Design | Google Stitch | UI 디자인 생성 |
| Implement | `gpt-5.3-codex-high` | 코드 구현 + PR |
| Verify | `gpt-5.3-codex-high` | 테스트·린트·빌드 자동 검증 |
| Debug | `gpt-5.3-codex-high` | 검증 실패 시 자동 디버깅 (최대 3회 루프) |
| SecurityPatch | `claude-fable-5-thinking-high` | 보안 감사 + 취약점 자동 패치 |
| Deliver | — | 산출물 패키징 |

## 품질 게이트 워크플로우

```
Implement → Verify ──(실패)──→ Debug ──→ Verify (최대 3회)
                ↓ (통과)
         SecurityPatch → Deliver
```

- **Verify**: `cargo check/test/clippy`, 린트 실행 → `verify_report.json`
- **Debug**: 실패 원인 분석·수정 → `debug_report.json` → Verify 재실행
- **SecurityPatch**: `cargo audit`, OWASP 점검, 의존성 패치 → `security_report.json`

환경 변수 `MAX_DEBUG_CYCLES` (기본 3)로 디버깅 재시도 횟수 조절.

## Podman + 메시지 큐 + Slack

분산 배포는 [docs/PODMAN.md](docs/PODMAN.md) 참고.

```bash
export SLACK_WEBHOOK_URL=https://hooks.slack.com/services/...
./scripts/podman-up.sh
```

- **Podman**: API / Orchestrator / Worker 컨테이너 분리
- **Redis Streams**: 커맨드·이벤트 큐로 Worker 수평 확장
- **Slack**: 스테이지별 진행률 실시간 알림

## 프로그램 구조

```
src/
├── main.rs              # CLI 진입점 (기본: 웹 서버)
├── lib.rs               # 내부 모듈 루트
├── app.rs               # 애플리케이션 상태
├── config.rs            # 환경 변수 설정
├── domain/              # 도메인 타입
├── clients/             # Cursor API, Stitch MCP
├── services/            # ingest, orchestrator, worker, pipeline
└── web/                 # Actix-web 라우트·핸들러
static/
└── index.html           # 웹 대시보드 (PDF 업로드 UI)
```

## 실행 방법

```bash
# 빌드
cargo build --release

# 환경 변수
export CURSOR_API_KEY=your_key
export STITCH_API_KEY=your_key

# 웹 서버 시작 (기본 명령)
cargo run
# 또는
cargo run -- serve --port 8080

# 브라우저에서 접속
open http://localhost:8080
```

## API 엔드포인트 (Actix-web)

| Method | Path | 설명 |
|--------|------|------|
| GET | `/` | 웹 대시보드로 리다이렉트 |
| GET | `/health` | 헬스체크 |
| GET | `/static/index.html` | 업로드 UI |
| POST | `/v1/projects` | PDF 업로드 + 파이프라인 시작 (multipart) |
| GET | `/v1/projects` | 프로젝트 목록 |
| GET | `/v1/projects/{id}` | 프로젝트 상태 |
| GET | `/v1/projects/{id}/stream` | SSE 진행률 |
| POST | `/v1/projects/{id}/cancel` | 취소 |

### PDF 업로드 예시

```bash
curl -X POST http://localhost:8080/v1/projects \
  -F "name=테스트 프로젝트" \
  -F "repo_url=https://github.com/org/repo" \
  -F "plan=@plan.pdf"
```

## 환경 변수

| 변수 | 기본값 | 설명 |
|------|--------|------|
| `HOST` | `0.0.0.0` | 바인드 호스트 |
| `PORT` | `8080` | 포트 |
| `CURSOR_API_KEY` | — | Cursor Cloud Agents API |
| `STITCH_API_KEY` | — | Google Stitch MCP |
| `ARTIFACTS_ENDPOINT` | `http://localhost:9000` | S3/MinIO 엔드포인트 |
| `ARTIFACTS_BUCKET` | `autoforge` | 버킷 이름 |
| `DEFAULT_REPO_URL` | — | 구현 단계 기본 repo |
| `MESSAGE_QUEUE_ENABLED` | `false` (로컬) | Redis MQ 활성화 |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis 연결 |
| `SLACK_WEBHOOK_URL` | — | Slack 진행률 Webhook |
| `SLACK_BOT_TOKEN` | — | Slack Bot (스레드용) |
| `SLACK_CHANNEL` | — | Slack 채널 ID |
| `MAX_DEBUG_CYCLES` | `3` | Debug 재시도 횟수 |

## 상세 문서

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — 아키텍처 설계
- [docs/PLANNING.md](docs/PLANNING.md) — 구현 로드맵

## 라이선스

Apache-2.0
