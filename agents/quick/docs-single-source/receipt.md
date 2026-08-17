# Receipt - docs-single-source

## Goal

Make `README.md` the single canonical install and configuration reference, reduce `INSTALL.md` to a Korean quick start that defers to it, correct every factual error the documentation review found, document the surfaces that were missing entirely, and add tests that fail when a documented example drifts from the code.

## Outcome

PASS - all eight acceptance criteria judged and passed on the first verify attempt, with `cargo test` (44 passed), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and `cargo build` all green.

The two new drift guards were negative-tested by hand before verification: removing a token from the README example, giving working and done the same color, and renaming the model each made the corresponding test fail. The first guard was rewritten after that check exposed it as vacuous - it originally searched the whole README, so prose naming a token elsewhere satisfied it; it now scopes to the configuration block itself.

## Verification

`sasu gate verify --slug docs-single-source --contract agents/quick/docs-single-source/contract.md --base a4969bf574d80b895da1fcc8ee868ed413a30a56 --json`, verbatim (command tails truncated for length):

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
        "source": "config",
        "criterionIds": [
          "AC2",
          "AC3"
        ],
        "exitCode": 0,
        "ok": true,
        "tail": "test tests::a_repeating_failure_is_abandoned_instead_of_retried_forever ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\ntest tests::failures_are_logged_with_actionable_detail ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_call_the_request_spacer_defers_is_still_made_afterwards ... ok\ntest tests::an_attention\n… (trimmed)"
      },
      {
        "kind": "lint",
        "command": "cargo clippy --all-targets -- -D warnings",
        "source": "config",
        "exitCode": 0,
        "ok": true,
        "tail": "Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s"
      },
      {
        "kind": "typecheck",
        "command": "cargo fmt --check",
        "source": "config",
        "exitCode": 0,
        "ok": true,
        "tail": ""
      },
      {
        "kind": "build",
        "command": "cargo build",
        "source": "config",
        "exitCode": 0,
        "ok": true,
        "tail": "Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s"
      }
    ],
    "resolved": [
      {
        "kind": "test",
        "command": "cargo test",
        "source": "config",
        "criterionIds": [
          "AC2",
          "AC3"
        ]
      },
      {
        "kind": "lint",
        "command": "cargo clippy --all-targets -- -D warnings",
        "source": "config"
      },
      {
        "kind": "typecheck",
        "command": "cargo fmt --check",
        "source": "config"
      },
      {
        "kind": "build",
        "command": "cargo build",
        "source": "config"
      }
    ],
    "configSuggestion": null
  },
  "criteria": [
    {
      "id": "AC1",
      "verdict": "PASS",
      "reason": "README.md explicitly declares itself canonical and INSTALL.md redirects detailed configuration and troubleshooting guidance to it.",
      "evidence": "README.md Install section; INSTALL.md opening paragraph"
    },
    {
      "id": "AC2",
      "verdict": "PASS",
      "reason": "The sidebar example includes all published status tokens, and the passing cargo test verifies complete token coverage plus distinct working and done colors.",
      "evidence": "README.md Sidebar layout; src/tests.rs the_documented_sidebar_example_colors_every_token_the_plugin_publishes; AC2 harness result"
    },
    {
      "id": "AC3",
      "verdict": "PASS",
      "reason": "The passing test requires the README model to equal the code's MODEL constant and rejects free-model claims in README.md and INSTALL.md.",
      "evidence": "src/tests.rs the_documented_model_is_the_one_the_code_calls; AC3 harness result; INSTALL.md OpenRouter section"
    },
    {
      "id": "AC4",
      "verdict": "PASS",
      "reason": "The Privacy section states that analysis context spans the last two user turns.",
      "evidence": "README.md Privacy section"
    },
    {
      "id": "AC5",
      "verdict": "PASS",
      "reason": "README.md documents the config path, override behavior, partial-list ordering, watcher restart requirement, and a default order beginning with error.",
      "evidence": "README.md Sort order section"
    },
    {
      "id": "AC6",
      "verdict": "PASS",
      "reason": "README.md defines unread versus seen panes and maps the three question, approval, and error _new tokens to unread symbols.",
      "evidence": "README.md Unread and seen section"
    },
    {
      "id": "AC7",
      "verdict": "PASS",
      "reason": "README.md states that approval can come from Herdr's blocked lifecycle without hooks and that error requires the StopFailure hook.",
      "evidence": "README.md Where each symbol comes from and Agent hooks sections"
    },
    {
      "id": "AC8",
      "verdict": "PASS",
      "reason": "The obsolete analysis_fingerprint instruction was removed, and analyze-stdin is documented immediately after verify-live-provider.",
      "evidence": "INSTALL.md troubleshooting deletion; README.md verification commands and troubleshooting section"
    }
  ],
  "inputs": [
    {
      "path": "agents/quick/docs-single-source/contract.md",
      "sha256": "ba7ed4f418d47c13fc526fee94ef11d56f9964fe5e65996b4df15aafbe29d0cd"
    },
    {
      "path": "agents/config.json",
      "sha256": "7ac3c59e872e05e787e9a31e367350a00f61db811a290d0cc1eff01454d83917",
      "kind": "config"
    }
  ],
  "evidence": [],
  "checks": [
    {
      "criterionId": "AC2",
      "command": "cargo test",
      "exitCode": 0,
      "tail": "test tests::a_repeating_failure_is_abandoned_instead_of_retried_forever ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\ntest tests::failures_are_logged_with_actionable_detail ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_call_the_request_spacer_defers_is_still_made_afterwards ... ok\ntest tests::an_attention\n… (trimmed)"
    },
    {
      "criterionId": "AC3",
      "command": "cargo test",
      "exitCode": 0,
      "tail": "test tests::a_repeating_failure_is_abandoned_instead_of_retried_forever ... ok\ntest tests::a_failed_turn_is_retired_once_the_agent_runs_again ... ok\ntest tests::failures_are_logged_with_actionable_detail ... ok\ntest tests::session_path_prefers_the_herdr_reported_identity ... ok\ntest tests::a_call_the_request_spacer_defers_is_still_made_afterwards ... ok\ntest tests::an_attention\n… (trimmed)"
    }
  ],
  "judgedCriteriaIds": [
    "AC1",
    "AC2",
    "AC3",
    "AC4",
    "AC5",
    "AC6",
    "AC7",
    "AC8"
  ],
  "judgedVerdict": "PASS"
}
```
