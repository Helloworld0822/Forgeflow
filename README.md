# AutoForge — AI 외주 자동화 오케스트레이터

PDF 계획서를 입력받아 **요약(Sonnet) → 아키텍처·기획(Fable) → 디자인(Stitch) → 구현(Codex 5.3)** 파이프라인을 자동 실행하는 Rust 기반 오케스트레이션 시스템입니다.

## 모델 역할 분담

| 단계 | 모델 | 실행 환경 |
|------|------|-----------|
| PDF 요약 | `claude-4.6-sonnet-high-thinking` | Cursor Cloud Agent |
| 아키텍처·기획 | `claude-fable-5-thinking-high` | Cursor Cloud Agent (`mode: plan`) |
| UI 디자인 | Google Stitch (Gemini) | Stitch MCP API |
| 코딩 | `gpt-5.3-codex-high` | Cursor Cloud Agent (`mode: agent`) |

## 아키텍처 개요

```
[Client] → [API Gateway] → [Orchestrator] → [Stage Workers]
                                    ↓
                            [PostgreSQL + Redis + S3]
                                    ↓
                    [Cursor API]  [Stitch MCP]
```

자세한 설계는 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), 구현 로드맵은 [docs/PLANNING.md](docs/PLANNING.md)를 참고하세요.

## Crate 구조

```
crates/
├── shared/          # 공통 타입, 이벤트, 에러
├── cursor-client/   # Cursor Cloud Agents API v1 클라이언트
├── stitch-client/   # Google Stitch MCP 클라이언트
├── ingest/          # PDF 파싱·검증
├── artifacts/       # 산출물 저장 (S3 호환)
├── orchestrator/    # 상태 머신·DAG 스케줄러
├── worker/          # 스테이지 실행기
├── api/             # REST API (Axum)
└── cli/             # 바이너리 진입점
```

## 빠른 시작 (개발)

```bash
# 의존성 확인 (Rust 1.78+)
cargo check

# 환경 변수
export CURSOR_API_KEY=...
export STITCH_API_KEY=...
export DATABASE_URL=postgres://...
export REDIS_URL=redis://...
export ARTIFACTS_BUCKET=autoforge-artifacts
```

## 라이선스

Apache-2.0
