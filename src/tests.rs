use super::*;
use std::cell::RefCell;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::tempdir;

/// Block until the background analysis thread has produced its result, then put
/// it back for the next scan to consume. Waiting on the channel is what makes
/// the provider tests deterministic instead of sleep-timed.
impl<T: HerdrTransport, C: AnalysisClient, R: SessionReader> Watcher<T, C, R> {
    fn await_pending_analysis(&mut self) {
        if self.analysis_in_flight.is_empty() {
            return;
        }
        if let Ok(outcome) = self.analysis_receiver.recv_timeout(Duration::from_secs(5)) {
            let _ = self.analysis_sender.send(outcome);
        }
    }
}

fn pane(id: &str, agent: AgentKind, status: &str) -> Pane {
    Pane {
        id: id.to_owned(),
        agent,
        agent_session: None,
        agent_status: status.to_owned(),
        revision: 1,
        state_change_seq: 1,
        cwd: None,
        focused: false,
    }
}

// ---------------------------------------------------------------- AC1

#[test]
fn agent_list_payload_from_a_live_session_parses_without_a_second_endpoint() {
    let payload = include_str!("../tests/fixtures/agent-list.json");
    let envelope: AgentListEnvelope = serde_json::from_str(payload).unwrap();
    let panes: Vec<Pane> = envelope
        .result
        .agents
        .into_iter()
        .filter_map(AgentListItem::into_pane)
        .collect();

    assert!(!panes.is_empty(), "live payload has no supported panes");
    // Unsupported kinds (hermes) are dropped rather than erroring the scan.
    assert!(
        panes
            .iter()
            .all(|pane| matches!(pane.agent, AgentKind::Claude | AgentKind::Codex))
    );
    // Everything a scan needs comes from this one response.
    assert!(panes.iter().any(|pane| pane.agent_session.is_some()));
    assert!(panes.iter().any(|pane| pane.cwd.is_some()));
    assert!(panes.iter().all(|pane| pane.state_change_seq > 0));
    assert_eq!(panes.iter().filter(|pane| pane.focused).count(), 1);
}

// ---------------------------------------------------------------- AC2

#[test]
fn session_path_prefers_the_herdr_reported_identity() {
    let root = tempdir().unwrap();
    let project = root.path().join(".claude/projects/-Users-example");
    let other = root
        .path()
        .join(".claude/projects/-Users-example-projects-other");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&other).unwrap();
    let reported = "54165405-3108-474a-beab-547903c6c23d";
    fs::write(project.join(format!("{reported}.jsonl")), "").unwrap();
    fs::write(other.join("11111111-0000-0000-0000-000000000000.jsonl"), "").unwrap();

    let reader = LocalSessionReader::new(root.path());
    let mut target = pane("w1:p1", AgentKind::Claude, "idle");
    target.cwd = Some("/Users/example".to_owned());
    target.agent_session = Some(AgentSession::new("id", reported));

    assert_eq!(
        reader.session_path(&target).unwrap(),
        project.join(format!("{reported}.jsonl"))
    );
}

#[test]
fn session_path_falls_back_to_the_newest_file_for_the_working_directory() {
    let root = tempdir().unwrap();
    let project = root.path().join(".claude/projects/-Users-example");
    fs::create_dir_all(&project).unwrap();
    let older = project.join("aaaaaaaa-0000-0000-0000-000000000000.jsonl");
    let newer = project.join("bbbbbbbb-0000-0000-0000-000000000000.jsonl");
    fs::write(&older, "").unwrap();
    std::thread::sleep(Duration::from_millis(20));
    fs::write(&newer, "").unwrap();

    let reader = LocalSessionReader::new(root.path());
    let mut target = pane("w1:p1", AgentKind::Claude, "idle");
    target.cwd = Some("/Users/example".to_owned());

    assert_eq!(reader.session_path(&target).unwrap(), newer);
}

#[test]
fn codex_session_falls_back_through_the_recent_day_directories() {
    let root = tempdir().unwrap();
    let day = root.path().join(".codex/sessions/2026/08/14");
    fs::create_dir_all(&day).unwrap();
    let mine = day.join("rollout-2026-08-14T10-00-00-aaaa.jsonl");
    let theirs = day.join("rollout-2026-08-14T11-00-00-bbbb.jsonl");
    // A real session_meta line embeds the full base instructions and runs to
    // tens of kilobytes, which is what broke a fixed-size head read.
    let meta = |cwd: &str| {
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"cwd\":\"{cwd}\",\"base_instructions\":\"{}\"}}}}\n",
            "instruction ".repeat(4_000)
        )
    };
    fs::write(&theirs, meta("/Users/example/projects/theirs")).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    fs::write(&mine, meta("/Users/example/projects/mine")).unwrap();
    assert!(fs::metadata(&mine).unwrap().len() > 16 * 1024);

    let reader = LocalSessionReader::new(root.path());
    let mut target = pane("w1:p1", AgentKind::Codex, "idle");
    target.cwd = Some("/Users/example/projects/mine".to_owned());

    assert_eq!(reader.session_path(&target).unwrap(), mine);
}

#[test]
fn session_reads_only_the_tail_of_a_large_file() {
    let root = tempdir().unwrap();
    let path = root.path().join("session.jsonl");
    let filler = "{\"type\":\"user\",\"message\":{\"content\":\"오래된 내용\"}}\n";
    let mut contents = filler.repeat(20_000);
    contents.push_str("{\"type\":\"user\",\"message\":{\"content\":\"최신 요청\"}}\n");
    fs::write(&path, &contents).unwrap();
    assert!(fs::metadata(&path).unwrap().len() > SESSION_TAIL_BYTES);

    let tail = read_tail(&path, SESSION_TAIL_BYTES).unwrap();

    assert!(tail.len() as u64 <= SESSION_TAIL_BYTES);
    assert!(tail.contains("최신 요청"));
    // The first line of a tail read is cut in half and must be discarded.
    assert!(serde_json::from_str::<serde_json::Value>(tail.lines().next().unwrap()).is_ok());
}

// ---------------------------------------------------------------- AC3

#[test]
fn skips_malformed_session_lines() {
    let torn = concat!(
        r#"{"type":"user","message":{"content":"첫 번째 요청"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"content":[{"type":"text","text":"작업했습니다."}]}}"#,
        "\n",
        r#"{"type":"assistant","message":{"content":[{"type":"tex"#,
    );

    let parsed = parse_claude_events(torn);

    assert_eq!(parsed.skipped_lines, 1);
    assert_eq!(parsed.events.len(), 2);
    assert_eq!(parsed.events[0].text, "첫 번째 요청");
    assert_eq!(parsed.events[1].text, "작업했습니다.");

    let codex = concat!(
        r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"요약을 고쳐줘"}]}}"#,
        "\n",
        r#"{"type":"response_item","payload":{"type":"messa"#,
    );
    let parsed = parse_codex_events(codex);
    assert_eq!(parsed.skipped_lines, 1);
    assert_eq!(parsed.events.len(), 1);
}

// ---------------------------------------------------------------- AC4

#[test]
fn redaction_preserves_prose_and_bullets() {
    let events = [SessionEvent {
        role: "assistant",
        text: concat!(
            "세 문제 모두 대기 중입니다. 답 주시면 채점해 드릴게요.\n",
            "- 문제 1 물 제외 가장 많이 소비되는 음료\n",
            "- 문제 2 노벨상에 없는 분야\n",
            "+ 추가 문항도 있습니다\n",
            "1-②, 2-④ 이런 식으로 주셔도 됩니다.\n",
            "api_key=supersecret\n",
            "user@example.com\n",
            "/Users/example/secret.txt\n",
            "```rust\n",
            "fn leak() { let secret = 1; }\n",
            "```",
        )
        .to_owned(),
    }];

    let context = analysis_context(&events);

    // The content the verdict depends on survives.
    assert!(context.contains("- 문제 1 물 제외 가장 많이 소비되는 음료"));
    assert!(context.contains("- 문제 2 노벨상에 없는 분야"));
    assert!(context.contains("+ 추가 문항도 있습니다"));
    assert!(context.contains("답 주시면 채점해 드릴게요"));
    // Secrets, personal data, paths and code do not.
    assert!(!context.contains("supersecret"));
    assert!(!context.contains("user@example.com"));
    assert!(!context.contains("/Users"));
    assert!(!context.contains("leak"));
}

// ---------------------------------------------------------------- AC5

#[test]
fn context_spans_the_last_two_user_turns() {
    let events = [
        SessionEvent {
            role: "user",
            text: "가장 오래된 요청입니다".to_owned(),
        },
        SessionEvent {
            role: "assistant",
            text: "가장 오래된 응답입니다".to_owned(),
        },
        SessionEvent {
            role: "user",
            text: "직전 요청입니다".to_owned(),
        },
        SessionEvent {
            role: "assistant",
            text: "직전 응답입니다".to_owned(),
        },
        SessionEvent {
            role: "user",
            text: "최신 요청입니다".to_owned(),
        },
        SessionEvent {
            role: "assistant",
            text: "긴 응답 ".repeat(200),
        },
        SessionEvent {
            role: "assistant",
            text: "마지막으로 답을 기다립니다".to_owned(),
        },
    ];

    let context = analysis_context(&events);

    assert!(context.contains("최신 요청입니다"));
    assert!(context.contains("마지막으로 답을 기다립니다"));
    // The previous exchange stays visible so a wrap-up of an already answered
    // question is not mistaken for a fresh one.
    assert!(context.contains("직전 요청입니다"));
    assert!(context.contains("직전 응답입니다"));
    // Anything older still stays out.
    assert!(!context.contains("가장 오래된 요청입니다"));
    assert!(!context.contains("가장 오래된 응답입니다"));
}

// ---------------------------------------------------------------- AC7

#[test]
fn done_status_comes_from_herdr() {
    assert_eq!(status_icon("done", None), StatusIcon::Done);
    assert_eq!(status_icon("idle", None), StatusIcon::Idle);
    assert_eq!(status_icon("working", None), StatusIcon::Working);
    // Herdr already knows a dialog is waiting for a key; that is `!`, not `?`.
    assert_eq!(status_icon("blocked", None), StatusIcon::Approval);
    assert_eq!(status_icon("unknown", None), StatusIcon::Stale);
    assert_eq!(StatusIcon::Working.symbol(true), "●");
    assert_eq!(StatusIcon::Working.symbol(false), "○");

    // The plugin keeps no unseen-completion state of its own: a pane reported
    // as done renders done, and the same pane reported as idle renders idle,
    // with no local memory in between.
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let mut watcher = Watcher::<_, FakeClient, _>::new(
        FakeTransport::new(vec![pane("w1:p5", AgentKind::Claude, "working")]),
        None,
        FakeSessionReader,
        paths,
    );
    watcher.scan().unwrap();

    watcher.transport.panes[0].agent_status = "done".to_owned();
    watcher.transport.panes[0].state_change_seq = 2;
    watcher.scan().unwrap();
    assert_eq!(watcher.last_report().status, StatusIcon::Done);

    watcher.transport.panes[0].agent_status = "idle".to_owned();
    watcher.transport.panes[0].state_change_seq = 3;
    watcher.scan().unwrap();
    assert_eq!(watcher.last_report().status, StatusIcon::Idle);
}

#[test]
fn attention_refines_the_herdr_state_instead_of_replacing_it() {
    // Herdr's own lifecycle is the base and is never overwritten by the two
    // things the plugin adds on top of it.
    for status in ["idle", "done", "blocked", "unknown"] {
        assert_eq!(
            status_icon(status, Some(Attention::Question)),
            StatusIcon::Question,
            "{status} should accept a question refinement"
        );
        assert_eq!(
            status_icon(status, Some(Attention::Error)),
            StatusIcon::Error,
            "{status} should accept an error refinement"
        );
    }

    // A running agent is waiting on nobody, so nothing refines it. Without this
    // guard a verdict drawn one turn ago repaints a pane that has moved on.
    assert_eq!(
        status_icon("working", Some(Attention::Question)),
        StatusIcon::Working
    );
    assert_eq!(
        status_icon("working", Some(Attention::Error)),
        StatusIcon::Working
    );
    assert_eq!(
        status_icon("working", Some(Attention::Approval)),
        StatusIcon::Working
    );

    // The hook sees a permission request before Herdr sees the dialog on screen.
    assert_eq!(
        status_icon("idle", Some(Attention::Approval)),
        StatusIcon::Approval
    );
    // `?` and `!` answer different questions: one needs your words, the other
    // needs a keypress.
    assert_eq!(StatusIcon::Question.symbol(true), "?");
    assert_eq!(StatusIcon::Approval.symbol(true), "!");
}

#[test]
fn a_failed_turn_is_retired_once_the_agent_runs_again() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let failure = serde_json::json!({ "hook_event_name": "StopFailure" });
    assert_eq!(
        apply_hook_payload(&paths, "w1:p1", &failure).unwrap(),
        HookUpdate::Set(Attention::Error)
    );

    let mut watcher = Watcher::<_, FakeClient, _>::new(
        FakeTransport::new(vec![pane("w1:p1", AgentKind::Claude, "idle")]),
        None,
        FakeSessionReader,
        paths.clone(),
    );
    watcher.scan().unwrap();
    assert_eq!(watcher.last_report().status, StatusIcon::Error);

    // The agent picked the work back up, so the old failure is history.
    watcher.transport.panes[0].agent_status = "working".to_owned();
    watcher.transport.panes[0].state_change_seq = 2;
    watcher.scan().unwrap();
    assert_eq!(watcher.last_report().status, StatusIcon::Working);
    assert_eq!(load_hook_states(&paths).panes["w1:p1"].attention, None);

    watcher.transport.panes[0].agent_status = "idle".to_owned();
    watcher.transport.panes[0].state_change_seq = 3;
    watcher.scan().unwrap();
    assert_eq!(watcher.last_report().status, StatusIcon::Idle);
}

// ---------------------------------------------------------------- AC8

#[test]
fn setting_automatic_summaries_is_idempotent() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    assert!(load_settings(&paths).automatic_summaries);

    // Herdr can dispatch one action several times for a single keypress, so the
    // same command applied repeatedly has to land on the same state.
    for _ in 0..3 {
        set_automatic_summaries(&paths, false).unwrap();
        assert!(!load_settings(&paths).automatic_summaries);
    }
    for _ in 0..3 {
        set_automatic_summaries(&paths, true).unwrap();
        assert!(load_settings(&paths).automatic_summaries);
    }
}

#[test]
fn corrupt_state_files_do_not_stop_the_watcher() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    fs::create_dir_all(&paths.root).unwrap();
    fs::write(paths.settings(), "{ this is not json").unwrap();
    fs::write(paths.display_state(), "]").unwrap();

    assert!(load_settings(&paths).automatic_summaries);
    let mut watcher = Watcher::<_, FakeClient, _>::new(
        FakeTransport::new(vec![pane("w1:p1", AgentKind::Claude, "idle")]),
        None,
        FakeSessionReader,
        paths.clone(),
    );
    assert_eq!(watcher.scan().unwrap(), 1);
    assert!(
        fs::read_to_string(paths.log())
            .unwrap()
            .contains("display_state_reset")
    );
}

// ---------------------------------------------------------------- AC9

#[test]
fn summary_truncates_on_a_word_boundary() {
    assert_eq!(truncate_summary("짧은 요약"), "짧은 요약");
    // "Commit benchmark PRD validation" used to render as "Commit benchmark PRD validatio".
    assert_eq!(
        truncate_summary("Commit benchmark PRD validation"),
        "Commit benchmark PRD…"
    );
    assert!(
        truncate_summary("Commit benchmark PRD validation")
            .chars()
            .count()
            <= MAX_SUMMARY_CHARS
    );
    // A single unbroken token still has to fit the budget.
    let unbroken = "a".repeat(50);
    assert_eq!(
        truncate_summary(&unbroken).chars().count(),
        MAX_SUMMARY_CHARS
    );
    assert!(truncate_summary(&unbroken).ends_with('…'));
}

#[test]
fn normalizes_only_safe_one_line_summaries() {
    assert_eq!(
        normalize_summary("  간단한 작업 요약  "),
        Some("간단한 작업 요약".into())
    );
    assert_eq!(normalize_summary("\n"), None);
    assert_eq!(
        normalize_summary("**`Compact task labels`**"),
        Some("Compact task labels".into())
    );
    assert_eq!(normalize_summary("bad\u{0000}"), None);
    assert_eq!(normalize_summary("ab"), None);
}

#[test]
fn parses_the_provider_structured_output_contract() {
    assert_eq!(
        parse_analysis(r#"{"summary":"수학 문제 출제 및 채점","attention":"none"}"#).unwrap(),
        Analysis {
            summary: "수학 문제 출제 및 채점".into(),
            attention: None,
        }
    );
    // A bare question verdict with no expected_reply field carries no statable
    // user action, so it downgrades like an empty one.
    assert_eq!(
        parse_analysis(r#"{"summary":"음료 소비량 퀴즈 풀이","attention":"question"}"#)
            .unwrap()
            .attention,
        None
    );
    // A question verdict needs a statable user action to survive.
    assert_eq!(
        parse_analysis(
            r#"{"expected_reply":"배포 진행 여부를 답한다","summary":"배포 진행 여부 확인","attention":"question"}"#
        )
        .unwrap()
        .attention,
        Some(Attention::Question)
    );
    // Question without an expected reply is a surface match and is downgraded.
    assert_eq!(
        parse_analysis(
            r#"{"expected_reply":"","summary":"새 작업 지시 대기","attention":"question"}"#
        )
        .unwrap()
        .attention,
        None
    );
    assert!(parse_analysis(r#"{"summary":"작업 승인","attention":"approval"}"#).is_err());
    assert!(parse_analysis(r#"{"summary":"상태 없는 응답"}"#).is_err());
}

// ---------------------------------------------------------------- AC10

#[test]
fn failures_are_logged_with_actionable_detail() {
    // A provider failure keeps its class instead of collapsing to one string.
    assert_eq!(
        provider_transport_error(ureq::Error::StatusCode(429)).to_string(),
        "provider_http_429"
    );

    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let mut watcher = Watcher::new(
        FakeTransport::new(vec![pane("w1:p1", AgentKind::Claude, "idle")]),
        Some(FailingClient),
        FakeSessionReader,
        paths.clone(),
    );
    watcher.scan().unwrap();
    watcher.await_pending_analysis();
    watcher.scan().unwrap();
    let log = fs::read_to_string(paths.log()).unwrap();
    assert!(log.contains("provider_http_429"), "log was: {log}");

    // A successful verdict records what it decided and what it decided it from.
    let mut watcher = Watcher::new(
        FakeTransport::new(vec![pane("w1:p2", AgentKind::Claude, "idle")]),
        Some(QuestionClient),
        FakeSessionReader,
        paths.clone(),
    );
    watcher.scan().unwrap();
    watcher.await_pending_analysis();
    watcher.scan().unwrap();
    let log = fs::read_to_string(paths.log()).unwrap();
    assert!(log.contains("attention=question"), "log was: {log}");
    assert!(log.contains("context_chars="), "log was: {log}");
    assert!(log.contains("context=00"), "log was: {log}");

    // A transport failure reaches the caller with its reason intact, which is
    // what the watch loop writes into the log.
    let mut broken = Watcher::<_, FakeClient, _>::new(
        BrokenTransport,
        None,
        FakeSessionReader,
        StatePaths::for_tests(root.path()),
    );
    let error = broken.scan().unwrap_err();
    assert!(format!("{error:#}").contains("herdr socket is unavailable"));
}

#[test]
fn log_is_private_and_has_no_content_fields() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    append_log(&paths, "credential_unavailable", None, None).unwrap();
    let data = fs::read_to_string(paths.log()).unwrap();
    assert!(data.contains("schema_version"));
    assert!(!data.contains("prompt"));
    assert!(!data.contains("response"));
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(paths.log()).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn retention_keeps_at_most_three_log_files() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    fs::create_dir_all(&paths.root).unwrap();
    for index in 0..4 {
        fs::write(paths.root.join(format!("events.{index}.jsonl")), "x").unwrap();
    }
    enforce_retention(&paths).unwrap();
    let count = fs::read_dir(paths.root)
        .unwrap()
        .flatten()
        .filter(|item| item.file_name().to_string_lossy().starts_with("events"))
        .count();
    assert_eq!(count, 3);
}

// ---------------------------------------------------------- attention rules

#[test]
fn hook_events_map_only_runtime_attention_to_status() {
    let payload = |event: &str, tool: Option<&str>| {
        let mut value = serde_json::json!({ "hook_event_name": event });
        if let Some(tool) = tool {
            value["tool_name"] = serde_json::Value::String(tool.to_owned());
        }
        value
    };
    assert_eq!(
        classify_hook_payload(&payload("PreToolUse", Some("AskUserQuestion"))),
        HookUpdate::Set(Attention::Question)
    );
    assert_eq!(
        classify_hook_payload(&payload("PreToolUse", Some("functions.request_user_input"))),
        HookUpdate::Set(Attention::Question)
    );
    assert_eq!(
        classify_hook_payload(&payload("PermissionRequest", Some("Bash"))),
        HookUpdate::Set(Attention::Approval)
    );
    assert_eq!(
        classify_hook_payload(&payload("StopFailure", None)),
        HookUpdate::Set(Attention::Error)
    );
    assert_eq!(
        classify_hook_payload(&payload("PostToolUse", Some("Bash"))),
        HookUpdate::Clear
    );
}

#[test]
fn hook_completion_clears_only_the_matching_pending_tool() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let pending = serde_json::json!({
        "hook_event_name": "PermissionRequest",
        "tool_name": "Bash",
        "tool_use_id": "tool-2"
    });
    assert_eq!(
        apply_hook_payload(&paths, "w1:p1", &pending).unwrap(),
        HookUpdate::Set(Attention::Approval)
    );

    let unrelated = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Read",
        "tool_use_id": "tool-1"
    });
    assert_eq!(
        apply_hook_payload(&paths, "w1:p1", &unrelated).unwrap(),
        HookUpdate::Ignore
    );
    assert_eq!(
        load_hook_states(&paths).panes["w1:p1"].attention,
        Some(Attention::Approval)
    );

    let matching = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_use_id": "tool-2"
    });
    assert_eq!(
        apply_hook_payload(&paths, "w1:p1", &matching).unwrap(),
        HookUpdate::Clear
    );
    assert_eq!(load_hook_states(&paths).panes["w1:p1"].attention, None);
}

#[test]
fn a_cleared_hook_retires_an_older_semantic_verdict() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let subject = pane("w1:p1", AgentKind::Claude, "idle");
    let mut watcher = Watcher::<_, FakeClient, _>::new(
        FakeTransport::new(vec![subject.clone()]),
        None,
        FakeSessionReader,
        paths,
    );
    watcher.display_states.panes.insert(
        subject.id.clone(),
        PersistedDisplayState {
            semantic_attention: Some(Attention::Question),
            analysis_unix_ms: 1_000,
            ..PersistedDisplayState::default()
        },
    );

    // No hook has spoken: the inference stands.
    assert_eq!(
        watcher.resolve_attention(&subject),
        Some((Attention::Question, AttentionSource::Semantic))
    );

    // The hook says the interaction ended after that inference was drawn.
    watcher.hook_states.panes.insert(
        subject.id.clone(),
        HookState {
            attention: None,
            updated_unix_ms: 2_000,
            ..HookState::default()
        },
    );
    assert_eq!(watcher.resolve_attention(&subject), None);

    // A newer inference may speak again.
    watcher
        .display_states
        .panes
        .get_mut(&subject.id)
        .unwrap()
        .analysis_unix_ms = 3_000;
    assert_eq!(
        watcher.resolve_attention(&subject),
        Some((Attention::Question, AttentionSource::Semantic))
    );

    // A live hook signal always wins.
    watcher
        .hook_states
        .panes
        .get_mut(&subject.id)
        .unwrap()
        .attention = Some(Attention::Approval);
    assert_eq!(
        watcher.resolve_attention(&subject),
        Some((Attention::Approval, AttentionSource::Hook))
    );
}

#[test]
fn plain_text_question_survives_a_watcher_restart() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let subject = pane("w1:p5", AgentKind::Claude, "idle");
    let mut watcher = Watcher::new(
        FakeTransport::new(vec![subject.clone()]),
        Some(QuestionClient),
        FakeSessionReader,
        paths.clone(),
    );
    watcher.scan().unwrap();
    watcher.await_pending_analysis();
    watcher.scan().unwrap();
    assert_eq!(watcher.last_report().status, StatusIcon::Question);
    drop(watcher);

    let mut restarted = Watcher::<_, QuestionClient, _>::new(
        FakeTransport::new(vec![subject]),
        None,
        FakeSessionReader,
        paths,
    );
    restarted.scan().unwrap();
    assert_eq!(restarted.last_report().status, StatusIcon::Question);
}

// ------------------------------------------------------------- scheduling

#[test]
fn watcher_deduplicates_reports_and_its_own_revision_bump() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let mut watcher = Watcher::new(
        FakeTransport::new(vec![pane("w1:p1", AgentKind::Codex, "blocked")]),
        Some(FakeClient::default()),
        FakeSessionReader,
        paths,
    );
    // First report, then the verdict lands and is reported once more.
    assert_eq!(watcher.scan().unwrap(), 1);
    watcher.await_pending_analysis();
    assert_eq!(watcher.scan().unwrap(), 1);
    assert_eq!(watcher.client.as_ref().unwrap().calls(), 1);

    // Steady state: an unchanged pane produces no traffic at all.
    assert_eq!(watcher.scan().unwrap(), 0);

    // Herdr bumps the revision because of our own report; that must not look
    // like a new event, and must not spend another provider request.
    let own_bump = watcher.reported_revisions["w1:p1"];
    watcher.transport.panes[0].revision = own_bump;
    assert_eq!(watcher.scan().unwrap(), 0);
    assert_eq!(watcher.client.as_ref().unwrap().calls(), 1);
}

#[test]
fn a_changed_session_is_analyzed_even_inside_the_rate_limit_window() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let mut watcher = Watcher::new(
        FakeTransport::new(vec![pane("w1:p5", AgentKind::Claude, "idle")]),
        Some(FakeClient::default()),
        StateSequenceReader,
        paths,
    );
    watcher.scan().unwrap();
    watcher.await_pending_analysis();
    watcher.scan().unwrap();
    assert_eq!(watcher.client.as_ref().unwrap().calls(), 1);

    watcher.last_provider_request_at = Some(SystemTime::now() - Duration::from_secs(3));
    watcher.transport.panes[0].state_change_seq = 2;
    watcher.scan().unwrap();
    watcher.await_pending_analysis();
    watcher.scan().unwrap();
    assert_eq!(watcher.client.as_ref().unwrap().calls(), 2);
}

#[test]
fn a_refresh_request_is_consumed_once_by_the_focused_pane() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    set_automatic_summaries(&paths, false).unwrap();
    let mut focused = pane("w1:p5", AgentKind::Claude, "idle");
    focused.focused = true;
    let mut watcher = Watcher::new(
        FakeTransport::new(vec![focused]),
        Some(FakeClient::default()),
        FakeSessionReader,
        paths.clone(),
    );
    watcher.scan().unwrap();

    request_refresh(&paths).unwrap();
    assert!(paths.refresh_request().exists());
    watcher.scan().unwrap();

    assert!(!paths.refresh_request().exists());
    assert!(
        fs::read_to_string(paths.log())
            .unwrap()
            .contains("summary_refresh_skipped_disabled")
    );
    assert_eq!(watcher.client.as_ref().unwrap().calls(), 0);
}

#[test]
fn metadata_clears_every_status_token_it_may_own() {
    let subject = pane("w1:p1", AgentKind::Codex, "idle");
    let args = metadata_arguments(
        &subject,
        &Display {
            summary: Some("작업 요약".into()),
            status: StatusIcon::Approval,
            sort_key: SortKey::Approval,
            elapsed: Some("7s".into()),
            ..Display::default()
        },
    );

    assert!(args.iter().any(|item| item == "summary=작업 요약"));
    assert!(args.iter().any(|item| item == "status_approval=!"));
    assert!(args.iter().any(|item| item == "agent_codex=⬢"));
    assert!(args.iter().any(|item| item == "elapsed=7s"));
    // Approval sits in the user-blocking group at the top of the ordering.
    // The rank digit depends on the machine's optional sort-order file, so
    // only the seen partition prefix is asserted.
    assert!(
        args.iter()
            .any(|item| item.starts_with("sort_rank=1") && item.len() == "sort_rank=1x".len())
    );
    // Every other status token is cleared, so two icons can never render at
    // once; the one being set is not cleared to stay under the 16-token cap.
    for token in STATUS_TOKENS {
        if token == "status_approval" {
            continue;
        }
        assert!(
            args.windows(2)
                .any(|pair| pair[0] == "--clear-token" && pair[1] == token),
            "{token} is not cleared"
        );
    }
    // The report never exceeds Herdr's 16-token budget.
    let touched = args
        .iter()
        .filter(|a| *a == "--token" || *a == "--clear-token")
        .count();
    assert!(touched <= 16, "report touches {touched} tokens");
    // The pane title belongs to the user, not to this plugin.
    assert!(!args.iter().any(|item| item == "--title"));
}

#[test]
fn elapsed_time_uses_compact_second_minute_hour_and_day_units() {
    assert_eq!(format_elapsed(42_999), "42s");
    assert_eq!(format_elapsed(60_000), "1m");
    assert_eq!(format_elapsed(3_600_000), "1h");
    assert_eq!(format_elapsed(86_400_000), "1d");
}

#[test]
fn sort_rank_orders_user_blocking_states_before_ambient_states() {
    let order = resolve_sort_order(None);
    let ranks = DEFAULT_SORT_ORDER.map(|icon| sort_rank(&order, icon));
    assert_eq!(ranks, ["0", "1", "2", "3", "4", "5", "6", "7", "8"]);
    // The hook-confirmed question outranks the provider-inferred one.
    assert!(sort_rank(&order, SortKey::Question) < sort_rank(&order, SortKey::SemanticQuestion));
}

#[test]
fn user_sort_order_reorders_listed_states_and_appends_the_rest() {
    let order = resolve_sort_order(Some(r#"{"order":["working","question","nonsense"]}"#));
    assert_eq!(order[0], SortKey::Working);
    assert_eq!(order[1], SortKey::Question);
    // Unlisted states keep their default relative order after the listed ones.
    assert_eq!(order[2], SortKey::Approval);
    assert_eq!(order[8], SortKey::Stale);
    // Broken JSON must not take the watcher down or scramble the order.
    assert_eq!(resolve_sort_order(Some("not json")), DEFAULT_SORT_ORDER);
}

// ------------------------------------------------------------------ doubles

struct FakeTransport {
    panes: Vec<Pane>,
    reports: RefCell<Vec<Display>>,
}

impl FakeTransport {
    fn new(panes: Vec<Pane>) -> Self {
        Self {
            panes,
            reports: RefCell::new(Vec::new()),
        }
    }
}

impl HerdrTransport for FakeTransport {
    fn panes(&self) -> Result<Vec<Pane>> {
        Ok(self.panes.clone())
    }
    fn report(&self, _: &Pane, display: &Display) -> Result<()> {
        self.reports.borrow_mut().push(display.clone());
        Ok(())
    }
}

struct BrokenTransport;

impl HerdrTransport for BrokenTransport {
    fn panes(&self) -> Result<Vec<Pane>> {
        Err(anyhow!("herdr socket is unavailable"))
    }
    fn report(&self, _: &Pane, _: &Display) -> Result<()> {
        Ok(())
    }
}

impl<C: AnalysisClient, R: SessionReader> Watcher<FakeTransport, C, R> {
    fn last_report(&self) -> Display {
        self.transport.reports.borrow().last().cloned().unwrap()
    }
}

#[derive(Default)]
struct FakeClient {
    calls: AtomicUsize,
}

impl FakeClient {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl AnalysisClient for FakeClient {
    fn analyze(&self, _: &str) -> Result<Analysis> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Analysis {
            summary: "작업 요약".into(),
            attention: None,
        })
    }
}

struct QuestionClient;

impl AnalysisClient for QuestionClient {
    fn analyze(&self, _: &str) -> Result<Analysis> {
        Ok(Analysis {
            summary: "음료 소비량 퀴즈 풀이".into(),
            attention: Some(Attention::Question),
        })
    }
}

struct FailingClient;

impl AnalysisClient for FailingClient {
    fn analyze(&self, _: &str) -> Result<Analysis> {
        Err(provider_transport_error(ureq::Error::StatusCode(429)))
    }
}

struct FakeSessionReader;

impl SessionReader for FakeSessionReader {
    fn read(&self, _: &Pane) -> Result<ParsedSession> {
        Ok(ParsedSession {
            events: vec![SessionEvent {
                role: "user",
                text: "작업 요약을 생성하고 표시를 검증해줘".to_owned(),
            }],
            skipped_lines: 0,
        })
    }
}

/// A failure that repeats for the same input must stop being re-asked. Two idle
/// panes once spent 1520 provider calls overnight on a context that could never
/// succeed, which exhausted the daily budget before anyone was awake.
#[test]
fn a_repeating_failure_is_abandoned_instead_of_retried_forever() {
    let root = tempdir().unwrap();
    let paths = StatePaths::for_tests(root.path());
    let mut focused = pane("w1:p9", AgentKind::Claude, "idle");
    focused.focused = true;
    let mut watcher = Watcher::new(
        FakeTransport::new(vec![focused]),
        Some(MalformedClient),
        FakeSessionReader,
        paths.clone(),
    );

    // A refresh forces an attempt through immediately, standing in for the
    // wall-clock backoff the watch loop would otherwise wait out. Two of them
    // reach the cap for a failure that cannot change on its own.
    for _ in 0..PROVIDER_MAX_ATTEMPTS_TERMINAL {
        request_refresh(&paths).unwrap();
        watcher.scan().unwrap();
        watcher.await_pending_analysis();
        watcher.scan().unwrap();
    }
    // From here nobody asks for anything, and the watcher must stay quiet
    // rather than rediscovering the same failure on every poll.
    for _ in 0..8 {
        watcher.scan().unwrap();
        watcher.await_pending_analysis();
        watcher.scan().unwrap();
    }

    let log = fs::read_to_string(paths.log()).unwrap();
    let attempts = log.matches("provider_invalid_analysis").count();
    assert_eq!(
        attempts, PROVIDER_MAX_ATTEMPTS_TERMINAL as usize,
        "a settled failure retried {attempts} times: {log}"
    );
    assert!(log.contains("analysis_abandoned"), "log was: {log}");
    // Abandoning is not a verdict: nothing is claimed about the pane.
    assert!(!log.contains("attention="), "log was: {log}");
}

struct MalformedClient;

impl AnalysisClient for MalformedClient {
    fn analyze(&self, _: &str) -> Result<Analysis> {
        Err(anyhow::anyhow!("provider_invalid_analysis: expected value"))
    }
}

struct StateSequenceReader;

impl SessionReader for StateSequenceReader {
    fn read(&self, pane: &Pane) -> Result<ParsedSession> {
        Ok(ParsedSession {
            events: vec![SessionEvent {
                role: "user",
                text: format!("새 요청 {}", pane.state_change_seq),
            }],
            skipped_lines: 0,
        })
    }
}
