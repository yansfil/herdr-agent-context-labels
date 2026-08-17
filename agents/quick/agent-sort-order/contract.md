---
topic: agent-sort-order
status: complete
---

## Goal

Reorder the Herdr agent sidebar so panes rank by who is blocking whom, in the order the user specified: first work that finished and has not been looked at (error, then question/approval, then a plain completion), then work still running, then everything already seen.
Within every group, the most recently active pane comes first.
Three changes carry this: `Error` moves to the head of `DEFAULT_SORT_ORDER`, the seen partition collapses its status rank so nothing but recency orders it, and the view's recency tiebreak switches from the per-pane counter `state_change_seq` to a real wall-clock token, because a counter is only monotonic within one pane and cannot compare two panes.

## Non-goals

- Changing what makes a pane seen or unseen; focus-since-last-state-change stays the definition.
- Surfacing an error on a pane that is running: `Attention::Error` only originates from `StopFailure`, which fires when a turn ends, so the state is unreachable by construction.
- Changing colors, glyphs, or the Herdr-side theme config.

## Checks

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

## Acceptance Criteria

- AC1. `DEFAULT_SORT_ORDER` starts with `SortKey::Error`, placing it ahead of `Question`, `Approval`, and `SemanticQuestion`, and still contains all nine variants exactly once.
  - check: `cargo test`
- AC2. In `metadata_arguments`, a pane in the seen partition (not unseen and not `Working`) emits a `sort_rank` whose second character is a fixed constant, so no status rank differentiates seen panes and only the recency tiebreak orders them. Unseen and working panes keep their status rank.
  - check: `cargo test`
- AC3. `metadata_arguments` publishes an `activity` token holding the pane's last state-change time as epoch milliseconds, zero-padded to a fixed 13-character width so lexicographic and numeric comparison agree, and the token is always set rather than cleared.
  - check: `cargo test`
- AC4. `apply_priority_agent_view` requests the recency tiebreak as `{"field": {"token": "activity"}, "order": "desc"}` in place of the previous `state_change_seq` descending key, with the `sort_rank` ascending key still first.
  - check: `cargo test`
- AC5. The installed Herdr socket API accepts a token-valued sort field with a `desc` order, which is what AC4 depends on.
  - evidence: agents/quick/agent-sort-order/evidence/herdr-sort-schema.json
- AC6. `README.md` describes the resulting sidebar order (unseen error, unseen question/approval, unseen completion, working, then seen by recency) instead of the previous attention-first wording.
