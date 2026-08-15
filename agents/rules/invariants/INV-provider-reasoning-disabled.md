---
id: INV-provider-reasoning-disabled
kind: invariant
status: active
evidence:
  - "2026-08-15 세션: MODEL을 openai/gpt-5.6-luna로 교체 (commit 1f9aff7)"
trigger:
  paths:
    - "src/lib.rs"
check:
  type: grep
  pattern: '"reasoning": \{"enabled": false\}'
  files:
    - "src/lib.rs"
  expect: present
---

분석 요청은 max_tokens 96으로 한 줄짜리 분류를 시킨다. 추론이 켜진 모델은 그 96
토큰을 전부 추론에 쓰고 본문 없이 finish_reason "length"로 응답하므로, 파싱이
전건 실패한다. gpt-5.6-luna로 교체할 때 실제로 이 응답을 받았고, reasoning을 끈
뒤에야 정상 JSON이 나왔다.

MODEL 상수를 바꿀 때는 이 필드가 남아 있는지만 보지 말고, 실제 프롬프트로 라이브
호출을 한 번 해서 파싱 가능한 본문이 오는지 확인할 것. 응답 형식 실패는 재시도
캡에 걸려 조용히 "라벨 없음"으로 숨는다.
