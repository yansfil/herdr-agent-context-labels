# Receipt - agent-sort-order

## Goal

Reorder the Herdr agent sidebar to the user's scheme: unread finished work first (error, then question/approval, then a plain completion), then running work, then everything already seen, with the most recently active pane leading each group.

## Outcome

PASS - all six acceptance criteria judged and passed on the first verify attempt, with `cargo test`, `cargo build`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` all green.

## Verification

`sasu gate verify --slug agent-sort-order --contract agents/quick/agent-sort-order/contract.md --base d1fcc7a76d51c33a69aeb72261f401ed6d418b80 --json`, verbatim (command tails truncated for length):

```json
{
  "contractVersion": "0.5.0",
  "ok": true,
  "status": {
    "gate": "verify",
    "verdict": "PASS",
    "effective": "PASS",
    "stale": false,
    "inputsDrifted": false,
    "staleInputs": [],
    "overridden": false,
    "attempts": 0,
    "budget": 5,
    "budgetExhausted": false,
    "consecutiveErrors": 0,
    "judgeErrorLoop": false,
    "requiresHuman": false,
    "findings": [],
    "grants": 0,
    "assumedHumanFindings": 0
  },
  "prelint": {
    "ok": true,
    "doc": "contract",
    "findings": []
  },
  "mechanical": {
    "ok": true,
    "runs": [
      {
        "kind": "test",
        "command": "cargo test",
        "source": "detected",
        "criterionIds": [
          "AC1",
          "AC2",
          "AC3",
          "AC4"
        ],
        "exitCode": 0,
        "ok": true,
        "tail": "test tests::corrupt_state_files_do_not_stop_the_watcher ... ok\ntest tests::plain_text_question_survives_a_watcher_restart ... ok\ntest tests::done_status_comes_from_herdr ... ok\ntest tests::a_changed_session_is_analyzed_even_inside_the_rate_limit_window ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\nte\n… (trimmed)"
      },
      {
        "kind": "build",
        "command": "cargo build",
        "source": "detected",
        "exitCode": 0,
        "ok": true,
        "tail": "Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s"
      },
      {
        "kind": "check",
        "command": "cargo fmt --check",
        "source": "contract",
        "exitCode": 0,
        "ok": true,
        "tail": ""
      },
      {
        "kind": "check",
        "command": "cargo clippy --all-targets -- -D warnings",
        "source": "contract",
        "exitCode": 0,
        "ok": true,
        "tail": "Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s"
      }
    ],
    "resolved": [
      {
        "kind": "test",
        "command": "cargo test",
        "source": "detected",
        "criterionIds": [
          "AC1",
          "AC2",
          "AC3",
          "AC4"
        ]
      },
      {
        "kind": "build",
        "command": "cargo build",
        "source": "detected"
      },
      {
        "kind": "check",
        "command": "cargo fmt --check",
        "source": "contract"
      },
      {
        "kind": "check",
        "command": "cargo clippy --all-targets -- -D warnings",
        "source": "contract"
      }
    ],
    "configSuggestion": {
      "test": "cargo test",
      "build": "cargo build"
    }
  },
  "criteria": [
    {
      "id": "AC1",
      "verdict": "PASS",
      "reason": "`DEFAULT_SORT_ORDER` now begins with `SortKey::Error`, and the implementation test verifies each listed variant occurs exactly once.",
      "evidence": "src/lib.rs DEFAULT_SORT_ORDER hunk; src/tests.rs a_failed_turn_leads_the_default_order_ahead_of_every_question"
    },
    {
      "id": "AC2",
      "verdict": "PASS",
      "reason": "`sort_rank_token` assigns all seen non-working panes the fixed `10` rank while retaining status-derived ranks for unseen and working panes, and `metadata_arguments` publishes it.",
      "evidence": "src/lib.rs sort_rank_token and metadata_arguments hunks; src/tests.rs unread_work_ranks_by_attention_and_seen_work_only_by_recency"
    },
    {
      "id": "AC3",
      "verdict": "PASS",
      "reason": "`metadata_arguments` always emits `activity=` using `activity_token`, which zero-pads the pane state-change epoch milliseconds to 13 characters.",
      "evidence": "src/lib.rs activity_token, Display activity_unix_ms, Watcher display construction, and metadata_arguments hunks; src/tests.rs the_activity_token_is_a_fixed_width_clock_two_panes_can_be_compared_on"
    },
    {
      "id": "AC4",
      "verdict": "PASS",
      "reason": "The generated `agent.view.set` request keeps the `sort_rank` ascending key first and replaces `state_change_seq` with the `activity` token descending key.",
      "evidence": "src/lib.rs priority_agent_view_request hunk; src/tests.rs the_view_breaks_ties_on_the_activity_clock_not_the_per_pane_counter"
    },
    {
      "id": "AC5",
      "verdict": "PASS",
      "reason": "The registered Herdr schema explicitly permits token-valued sort fields and includes `desc` as a valid sort order.",
      "evidence": "agents/quick/agent-sort-order/evidence/herdr-sort-schema.json"
    },
    {
      "id": "AC6",
      "verdict": "PASS",
      "reason": "README.md states the order as unread error, unread question/approval, unread completion, running work, then seen panes with activity-based recency ties.",
      "evidence": "README.md sidebar ordering bullet hunk"
    }
  ],
  "inputs": [
    {
      "path": "agents/quick/agent-sort-order/contract.md",
      "sha256": "39154c21d2ae377b1a7b84d33ba4b121333327ad868fa0ab92b479592924474a"
    },
    {
      "path": "agents/config.json",
      "sha256": "sasu-gate-input-v2:absent",
      "kind": "config"
    },
    {
      "path": "agents/quick/agent-sort-order/evidence/herdr-sort-schema.json",
      "sha256": "2ef3120e5ad1ca8b8468b70b8419f8dd9ff3ae032e36e2a61e8f2686ff1ed75f",
      "kind": "evidence"
    }
  ],
  "evidence": [
    {
      "criterionId": "AC5",
      "path": "agents/quick/agent-sort-order/evidence/herdr-sort-schema.json",
      "sha256": "2ef3120e5ad1ca8b8468b70b8419f8dd9ff3ae032e36e2a61e8f2686ff1ed75f",
      "bytes": 1024
    }
  ],
  "checks": [
    {
      "criterionId": "AC1",
      "command": "cargo test",
      "exitCode": 0,
      "tail": "test tests::corrupt_state_files_do_not_stop_the_watcher ... ok\ntest tests::plain_text_question_survives_a_watcher_restart ... ok\ntest tests::done_status_comes_from_herdr ... ok\ntest tests::a_changed_session_is_analyzed_even_inside_the_rate_limit_window ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\nte\n… (trimmed)"
    },
    {
      "criterionId": "AC2",
      "command": "cargo test",
      "exitCode": 0,
      "tail": "test tests::corrupt_state_files_do_not_stop_the_watcher ... ok\ntest tests::plain_text_question_survives_a_watcher_restart ... ok\ntest tests::done_status_comes_from_herdr ... ok\ntest tests::a_changed_session_is_analyzed_even_inside_the_rate_limit_window ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\nte\n… (trimmed)"
    },
    {
      "criterionId": "AC3",
      "command": "cargo test",
      "exitCode": 0,
      "tail": "test tests::corrupt_state_files_do_not_stop_the_watcher ... ok\ntest tests::plain_text_question_survives_a_watcher_restart ... ok\ntest tests::done_status_comes_from_herdr ... ok\ntest tests::a_changed_session_is_analyzed_even_inside_the_rate_limit_window ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\nte\n… (trimmed)"
    },
    {
      "criterionId": "AC4",
      "command": "cargo test",
      "exitCode": 0,
      "tail": "test tests::corrupt_state_files_do_not_stop_the_watcher ... ok\ntest tests::plain_text_question_survives_a_watcher_restart ... ok\ntest tests::done_status_comes_from_herdr ... ok\ntest tests::a_changed_session_is_analyzed_even_inside_the_rate_limit_window ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\nte\n… (trimmed)"
    }
  ],
  "judgedCriteriaIds": [
    "AC1",
    "AC2",
    "AC3",
    "AC4",
    "AC5",
    "AC6"
  ],
  "judgedVerdict": "PASS"
}
```
