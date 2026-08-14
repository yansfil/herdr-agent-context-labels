# 설치 가이드

새 머신(로컬이든 원격 서버든)에서 이 플러그인을 처음부터 끝까지 적용하는 순서입니다.
모든 명령은 Herdr 서버가 돌고 있는 머신에서 실행합니다.

## 0. 요구사항

- Herdr 0.8.0 이상 (`herdr --version`)
- macOS 또는 Linux
- Rust stable 툴체인 (`cargo --version`) - GitHub 설치 시 빌드에 필요
- OpenRouter API 키 (요약과 질문 감지에 사용, 없어도 상태 심볼은 동작)

## 1. 플러그인 설치

```bash
herdr plugin install yansfil/herdr-agent-context-labels
```

에이전트 런타임별 Herdr 통합도 설치합니다 (lifecycle 상태의 원천).

```bash
herdr integration install claude
herdr integration install codex
```

## 2. OpenRouter 키 설정

Herdr 서버를 시작하는 환경에 키를 노출합니다.
서버가 키를 물려받지 못한 경우를 대비해, 워처 시작 스크립트가 `~/.zshrc`의 export 한 줄도 직접 읽습니다.

```bash
# ~/.zshrc
export OPENROUTER_API_KEY='sk-or-v1-...'
```

키 형식은 `sk-or-v1-` 접두사가 필수입니다.
무료 모델(`nvidia/nemotron-3-super-120b-a12b:free`)을 쓰므로 과금은 없지만, OpenRouter 계정 잔액이 $10 미만이면 무료 호출이 하루 50회로 제한될 수 있습니다.

## 3. 사이드바 레이아웃

플러그인은 토큰만 발행하고 배치와 색은 사용자 설정 소관입니다.
`~/.config/herdr/config.toml`에 추가:

```toml
[ui]
agent_panel_sort = "priority"

[ui.sidebar.agents]
rows = [
  [
    { token = "$status_question", fg = "#f9e2af", bold = true },
    { token = "$status_approval", fg = "#fab387", bold = true },
    { token = "$status_error", fg = "#f38ba8", bold = true },
    { token = "$status_working", fg = "#a6e3a1", bold = true },
    { token = "$status_done", fg = "#a6e3a1", bold = true },
    { token = "$status_interrupted", fg = "#cba6f7", bold = true },
    { token = "$status_idle", fg = "#a6adc8", bold = true },
    { token = "$status_stale", fg = "#6c7086", bold = true },
    "workspace",
    { token = "$agent_codex", fg = "#89b4fa", bold = true },
    { token = "$agent_claude", fg = "#fab387", bold = true },
    { token = "$elapsed", fg = "#6c7086", dim = true },
  ],
  [
    { token = "$summary", fg = "#74c7ec", bold = true },
  ],
]
```

적용:

```bash
herdr config check
herdr server reload-config
```

## 4. 워처 시작

플러그인의 startup hook이 Herdr 서버 시작 시 워처를 자동으로 띄웁니다.
서버가 이미 떠 있는 상태에서 방금 설치했다면, 서버를 재시작하거나 한 번만 수동으로 부트스트랩합니다.

```bash
root=$(herdr plugin list --plugin herdr-agent-context-labels --json \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['result']['plugins'][0]['plugin_root'])")
nohup /bin/sh "$root/scripts/start-watcher.sh" >/dev/null 2>&1 &
```

워처는 시작하면서 attention-first 정렬(`agent.view.set`)도 함께 설치합니다.
`agent_panel_sort` 정책을 덮어쓰는 transient view이며, 워처가 시작될 때마다 재적용됩니다.

## 5. (선택) 정밀 hook 등록

`?`(질문), `!`(승인), `×`(에러)를 도구 호출 수준에서 정확히 잡으려면 각 에이전트 런타임에 hook을 등록합니다.
등록하지 않아도 요약과 의미 기반 질문 감지는 동작합니다.

플러그인 루트를 확인한 뒤:

```text
/bin/sh '<plugin-root>/scripts/agent-hook.sh'
```

이 명령을 다음 이벤트에 기존 hook을 대체하지 말고 추가로 등록합니다.

| 이벤트 | 목적 |
| --- | --- |
| `PreToolUse` | `AskUserQuestion` / `request_user_input` 감지 |
| `PermissionRequest` | 승인 대기와 질문 구분 |
| `PostToolUse` / `PostToolUseFailure` | 대기 상태 해제 |
| `StopFailure` | 에러 표시 |
| `UserPromptSubmit` / `SessionStart` | 사용자 복귀 시 attention 해제 |

## 6. (선택) 정렬 순서 커스텀

기본 순서는 hook 질문 → hook 승인 → 의미 기반 질문 → 에러 → 미확인 완료 → 작업중 → idle → stale 입니다.
바꾸려면 플러그인 config 디렉터리에 `sort-order.json`을 만듭니다.

```bash
mkdir -p ~/.config/herdr/plugins/config/herdr-agent-context-labels
cat > ~/.config/herdr/plugins/config/herdr-agent-context-labels/sort-order.json <<'EOF'
{"order": ["question", "approval", "semantic_question", "error", "done", "working", "idle", "stale"]}
EOF
```

일부만 적으면 적은 항목이 위로 오고 나머지는 기본 상대 순서로 뒤에 붙습니다.
워처 재시작 시 반영됩니다.

## 7. 검증

```bash
# 플러그인과 액션 등록 확인
herdr plugin list --plugin herdr-agent-context-labels
herdr plugin action list --plugin herdr-agent-context-labels

# pane에 summary / status_* / sort_rank 토큰이 붙었는지 확인
herdr agent list

# 프로바이더 호출 1회 실검증 (pane 상태를 건드리지 않음)
"$root/target/release/herdr-agent-context-labels" verify-live-provider

# 임의 대화로 분류 결과 확인 (평가/디버깅용)
printf 'user: 빌드 돌려줘\nassistant: 빌드가 끝났습니다. 배포까지 진행할까요?' \
  | "$root/target/release/herdr-agent-context-labels" analyze-stdin
```

운영 이벤트 로그:

```bash
tail -n 50 ~/.local/state/herdr-agent-context-labels/events.jsonl
```

## 8. 트러블슈팅

| 증상 | 원인과 조치 |
| --- | --- |
| `credential_unavailable` | 키 미설정 또는 형식 오류. `~/.zshrc`의 export를 확인하고 워처 재시작 |
| `raw_session_unavailable` / `session_file_missing` | Herdr가 아는 세션과 실제 파일 불일치. 해당 pane에 메시지를 하나 보내면 새 세션으로 갱신됨 |
| `provider_http_429` | 무료 티어 rate limit. 잔액 $10 충전으로 한도 상향, 또는 `MODEL`을 유료 모델로 교체 후 재빌드 |
| 요약이 예전 것 그대로 | 요약은 pane 활동이 있을 때만 갱신됨. 강제 갱신은 `display-state.json`의 `analysis_fingerprint`를 null로 만들고 워처 재시작 |
| 두 워처가 동시에 돎 | 워처는 파일 잠금으로 단일 실행이 보장되므로 발생하지 않음. `watcher_already_running` 로그는 정상 |

## 참고: 원격 pane

이 플러그인은 로컬 세션 파일을 읽으므로, 원격 머신의 에이전트를 라벨링하려면 그 머신에 플러그인을 따로 설치해야 합니다.
mirror류 도구로 원격 pane을 로컬에 투영하는 경우, 원격 머신의 워처가 발행한 토큰이 함께 전달되는지는 해당 mirror 도구의 지원 여부에 달려 있습니다.
