# Rules Ledger

One row per learned rule. Bodies live at the landing path; this file is
metadata plus the evidence trail. Maintained by `rules add`; do not edit by hand.

| ID | Kind | Status | Landing | Evidence | Summary |
| --- | --- | --- | --- | --- | --- |
| FACT-watcher-deploy-topology | fact | active | `docs/deployment.md` | 2026-08-15 세션: pet pane status 디버깅 중 배포 경로 확인 | 실행 중인 워처는 이 저장소가 아니라 Herdr 플러그인 클론이고, startup 래퍼는 죽은 워처를 되살리지 않는다 |
| INV-provider-reasoning-disabled | invariant | active | `agents/rules/invariants/INV-provider-reasoning-disabled.md` | 2026-08-15 세션: MODEL을 openai/gpt-5.6-luna로 교체 (commit 1f9aff7) | 분석 요청은 max_tokens 96으로 한 줄짜리 분류를 시킨다. 추론이 켜진 모델은 그 96 토큰을 전부 추론에 쓰고 본문 없이 finish_reason "length"로 응답하므로, 파싱이 전건 실패한다. gpt-5.6-luna로 교체할 때 실제로 이 응답을 받았고, reasoning을 끈 뒤에야 정상 JSON이 나왔다. |
| REG-provider-retry-cap | regression | active | `src/tests.rs` | 2026-08-15 인시던트: 1581콜 실패 루프로 01:11에 1000 예산 소진 (commit 1f9aff7) | 확정적 provider 실패가 무한 재시도로 일일 예산을 태우지 않는지 검증한다 |
