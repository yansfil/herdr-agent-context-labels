# 설치 퀵스타트

새 머신에서 이 플러그인을 돌리기까지의 최단 경로입니다.
모든 명령은 Herdr 서버가 도는 머신에서 실행합니다.

정본 문서는 [README.md](README.md)입니다.
이 파일은 순서만 짚고, 상태 심볼의 의미, 사이드바 색 설정 전체, 정렬 순서 커스텀, 프라이버시, 트러블슈팅은 README를 보세요.
두 문서가 어긋나면 README가 맞습니다.

## 0. 요구사항

- Herdr 0.8.0 이상 (`herdr --version`)
- macOS 또는 Linux
- Rust stable 툴체인 (`cargo --version`) - GitHub 설치 시 빌드에 필요
- OpenRouter API 키 - 요약과 평문 질문 감지에 사용. 없어도 lifecycle 심볼은 그대로 동작합니다.

## 1. 플러그인과 통합 설치

```bash
herdr plugin install yansfil/herdr-agent-context-labels

herdr integration install claude
herdr integration install codex
```

Herdr 통합이 lifecycle 상태의 원천이라 사용하는 런타임마다 설치해야 합니다.

## 2. OpenRouter 키

Herdr 서버를 시작하는 환경에 키를 노출합니다.
서버가 값을 물려받지 못한 경우를 대비해 워처 시작 스크립트가 `~/.zshrc`의 export 한 줄도 직접 읽습니다.

```bash
# ~/.zshrc
export OPENROUTER_API_KEY='sk-or-v1-...'
```

호출은 유료 모델을 쓰므로 OpenRouter 계정에 잔액이 필요합니다.
사용량 상한은 README의 [Summary generation](README.md#summary-generation)을 보세요.

## 3. 사이드바 설정

플러그인은 토큰만 발행하고 배치와 색은 사용자 설정 소관입니다.
**전체 예시는 README의 [Sidebar layout](README.md#sidebar-layout)에 있고, 그대로 복사해 쓰면 됩니다.**

빠뜨리기 쉬운 두 가지만 여기 적어둡니다.

- 상태 토큰은 11개입니다. 미확인 상태를 나타내는 `_new` 변형 3개(`$status_error_new`, `$status_question_new`, `$status_approval_new`)를 빠뜨리면 정작 가장 급한 pane들이 색 없이 뜹니다.
- `$status_working`과 `$status_done`은 같은 `●` 기호라 반드시 다른 색을 줘야 합니다.

적용:

```bash
herdr config check
herdr server reload-config
```

## 4. 워처 시작

startup hook이 Herdr 서버 시작 시 워처를 자동으로 띄웁니다.
서버가 이미 떠 있는 상태에서 방금 설치했다면 서버를 재시작하거나 한 번만 수동으로 부트스트랩합니다.

```bash
root=$(herdr plugin list --plugin herdr-agent-context-labels --json \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['result']['plugins'][0]['plugin_root'])")
nohup /bin/sh "$root/scripts/start-watcher.sh" >/dev/null 2>&1 &
```

워처는 시작하면서 사이드바 정렬(`agent.view.set`)도 함께 설치합니다.
`agent_panel_sort` 정책을 덮어쓰는 transient view이며 워처가 시작될 때마다 재적용됩니다.

## 5. (선택) hook 등록

`×`(에러)는 hook을 등록해야만 나타납니다. `?`는 hook이 있으면 추측이 아니라 확정이 되고, `!`는 더 빨리 뜹니다.
등록 대상 이벤트와 명령은 README의 [Agent hooks](README.md#agent-hooks)에 있습니다.

## 6. 검증

```bash
herdr plugin list --plugin herdr-agent-context-labels
herdr agent list          # pane에 summary / status_* / sort_rank / activity 토큰이 붙었는지
tail -n 50 ~/.local/state/herdr-agent-context-labels/events.jsonl
```

증상별 원인과 조치는 README의 [Troubleshooting](README.md#troubleshooting)에 정리돼 있습니다.

## 참고: 원격 pane

이 플러그인은 로컬 세션 파일을 읽으므로, 원격 머신의 에이전트를 라벨링하려면 그 머신에 플러그인을 따로 설치해야 합니다.
mirror류 도구로 원격 pane을 로컬에 투영하는 경우, 원격 머신의 워처가 발행한 토큰이 함께 전달되는지는 해당 mirror 도구의 지원 여부에 달려 있습니다.
