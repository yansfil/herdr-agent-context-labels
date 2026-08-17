---
topic: "turn-keyed-analysis"
status: "ready"
human_approval: "pending"
review_profile: "standard"
review_rationale: "Changes when a paid external provider is called and when a pane's attention verdict may change, which alters runtime behavior and cost but touches no auth, persistent user data, or irreversible action."
source_intake: "current conversation"
created_at: "2026-08-17"
updated_at: "2026-08-17"
---

# PRD: turn-keyed-analysis

## 1. Summary

The watcher decides when to ask the LLM by hashing the text of the last two user turns and calling whenever that hash changes.
A live agent's transcript changes continuously, so this trigger fires without a natural bound: it is dammed only by a global two-second spacer, a ten-minute provider cooldown, and a thousand-request daily cap.
Measured on 2026-08-17 the watcher made 470 calls across 19 panes, 37 of them less than ten seconds apart on the same pane, and flipped its own attention verdict 158 times.
On 2026-08-15 the daily cap blew and the watcher spent the rest of the day writing 47,141 identical skip lines.

This PRD replaces the trigger rather than adding another throttle.
A call becomes keyed to **turn identity** - the last user message - instead of the churning text window.
A turn is a discrete, self-limiting event, so the analysis fires at most twice per turn per pane: once when the turn starts, to name the task, and once when the turn ends, to decide whether the agent is waiting for the user.
The attention verdict is then computed once per turn and held, which removes the flip-flop structurally instead of smoothing it.

Net effect on the codebase is a deletion: the content fingerprint disappears, and the throttles that existed to contain the unbounded trigger stop being load-bearing.

`Approval checklist`

- Scope boundary: trigger replacement only; the provider prompt, the hook signals, and the display/sort/color logic are untouched (section 3).
- Structure change: `analysis_fingerprint` is replaced by per-phase turn keys in the persisted display state, which is a state-file schema change (section 5).
- Behavior change: a pane's attention verdict no longer changes mid-turn, so a `?` raised by prose stays until the next turn boundary (R4, AC4).
- Retained backstops: the global spacer, provider cooldown, and daily cap stay as safety nets rather than being deleted alongside the fingerprint (section 3 non-goals, D5).
- Verification modes: automated behavior is required for done; live provider proof is not required (section 9.1).
- Delivery mode: local (section 4.3, D7).

## 2. Problem, Goal, And Users

The user runs many coding agents side by side in Herdr and reads the sidebar to decide which pane needs attention.
Two things make that sidebar untrustworthy today, and both come from the same trigger.

First, the attention symbol is unstable.
Each call independently re-classifies a slightly different slice of transcript with no memory of the previous verdict, so a pane can show `?` and then not show it and then show it again while nothing the user cares about has changed.
On 2026-08-17 the top offenders flipped 30, 23, and 22 times.
A symbol that changes on its own teaches the user to stop trusting it.

Second, the call volume is unbounded by design and is periodically catastrophic.
The trigger is "the text changed", and a running agent's text always changes.

The goal is a trigger whose firing rate is set by how the user actually works - one turn at a time - rather than by how fast an agent emits tokens.

Users: the single operator watching the Herdr sidebar. There is no multi-user or permission dimension.

### 2.1 User Scenarios

- SC1. A pane's label appears when work starts and stays put.
  Actors: the operator.
  Primary path: the operator submits a prompt to an agent pane; the sidebar shows a task summary for that pane while the agent works, and that summary does not churn as output streams.
  Failure state: the provider call for that turn fails; the pane keeps its previous summary rather than blanking or showing an error string.
  Recovery: the next turn the operator starts produces a fresh summary attempt.
  Reach: verification needs a pane whose session transcript grows across an agent turn; a synthetic session transcript driven through the watcher reaches this state.

- SC2. A waiting agent is marked once and stays marked.
  Actors: the operator.
  Primary path: an agent finishes a turn by asking a plain-prose question; the pane shows the question symbol from that moment until the operator acts on it, without the symbol appearing and disappearing on its own.
  Failure state: the provider judges the turn as needing nothing; the pane shows its ordinary completion state and does not later flip to a question without a new turn.
  Recovery: the operator answers, which starts a new turn and produces a new verdict.
  Reach: verification needs a session whose final assistant message is a question and a pane whose lifecycle leaves the working state; a synthetic session plus a simulated lifecycle transition reaches this state.

- SC3. A busy set of panes does not exhaust the day's provider budget.
  Actors: the operator.
  Primary path: several agents run long turns simultaneously; the number of provider calls tracks the number of turns taken, not the volume of output produced.
  Failure state: the daily cap is nonetheless reached; the watcher stops calling and keeps showing the last known labels rather than degrading the sidebar.
  Recovery: the cap resets the next day, or the operator resets the local counter.
  Reach: verification needs a driven sequence of turns and streaming output through the watcher with a counting fake provider.

## 3. Scope And Non-Goals

In scope:

- Replacing the context-hash analysis trigger with a turn-keyed trigger.
- Defining the two per-turn call points and the state that records them.
- Holding the attention verdict stable for the duration of a turn.
- Migrating the persisted display state off `analysis_fingerprint`.

Non-goals, each with its consequence:

- **Changing the provider prompt or model.** The classification itself is not being retuned; only its firing schedule changes. Consequence: a genuinely wrong verdict on a given turn stays wrong for that turn. Revisit if per-turn accuracy proves to be the dominant complaint once the flapping is gone.
- **Removing the global two-second spacer, the provider cooldown, or the daily cap.** They stop being load-bearing but remain as backstops against a defect or a pathological session. Consequence: three constants stay in the code that the new design does not strictly need. Revisit after a period of observed call volumes.
- **Changing hook-driven attention.** `PreToolUse`, `PermissionRequest`, and `StopFailure` remain exact, immediate, and unaffected by turn keying. Consequence: none; this is the reliable path and is deliberately preserved.
- **Changing the display, sort, or color logic.** Delivered separately in commit `cb3afee`. Consequence: none.
- **A per-pane minimum call interval and verdict hysteresis.** Explicitly rejected in conversation as knobs stacked on a broken trigger. Consequence: if turn keying somehow leaves residual churn, no smoothing exists to hide it - which is intended, because it would be evidence the trigger is still wrong.
- **Backfilling or migrating existing `display-state.json` entries.** The file is documented as a cache of what the panes already say. Consequence: on first run after upgrade, every pane analyzes once for its current turn. Acceptable and self-correcting.

## 4. Pre-Work And Required Decisions

### 4.1 Pre-Work Before Implementation

None required.
The provider key already exists in the runtime environment, the repository builds locally, and every verification mode planned here runs against fakes rather than the live provider.

### 4.2 Human Decisions Before PRD Approval

None required.
The `/please` invocation carries approval of the scope, the structure change, the verification modes, and local delivery.
No item in this PRD is in the hard-stop class: there are no credentials to issue, no billing decision, no production data, and no irreversible action.

### 4.3 Decision Traceability For Fidelity Review

- D1. User decision: adopt turn-number keying over content hashing, stated as "그렇게 깔끔하게 수정 부탁할게" in response to the proposed design. Represented by R1, AC1, T1.
- D2. User decision: reject the earlier three-part proposal (per-pane minimum interval, verdict hysteresis, narrowed fingerprint) as insufficiently clean - "헷갈리노. 단순하면서 깔끔하게 디자인을 할수는없는건가?". Represented as a non-goal in section 3.
- D3. User decision: abandon the earlier "idle dwell guard" proposal after evidence showed the three yellow panes were genuinely waiting and no false positive existed. Represented as a non-goal in section 3 and as context for R4.
- D4. Accepted proposal: two discrete call points per turn - summary at turn start, attention at turn end - rather than a single end-of-turn call, so a working pane still shows what it is doing. Represented by R2, AC2.
- D5. Agent assumption: keep the global spacer, provider cooldown, and daily cap rather than deleting them with the fingerprint. The user asked for a clean design and this leaves three now-redundant constants in place; the reasoning is that deleting a rate limiter is asymmetrically risky against a provider that bills. Represented as a non-goal in section 3. Vetoable.
- D6. Agent assumption: turn identity is the hash of the last user message text, not a counter of user messages. The session reader deliberately reads only the tail of a large file, so a whole-file count is not reliably available; the last user message is stable for the whole turn and changes exactly at a turn boundary. Represented by R1, AC1. Vetoable.
- D7. Agent assumption: delivery mode is local, because `agents/config.json` does not exist and the conversation agreed to a push only for the previously completed sort-order work. Represented in the Summary approval checklist and section 12. Vetoable.
- D8. Context-only fact: the running watcher is Herdr's own plugin clone, not this checkout, so no code change here reaches the live sidebar until the plugin is reinstalled. Recorded in section 10 as a risk, not as a task.
- D9. Agent assumption: the interrupted-turn path and the abandoned-analysis path should record the turn as analyzed for both phases, preserving their existing intent of not re-asking. Represented by R6, AC6. Vetoable.

## 5. Major Technical Structure Changes

- The persisted display state schema changes: `analysis_fingerprint: Option<u64>` is removed and replaced by two per-phase turn keys. `display-state.json` is a local runtime cache outside both checkouts, is already tolerant of corruption by rebuilding from scratch, and is not user data, so no migration path is required.
- The analysis trigger changes from content-derived to event-derived. This is a change in the watcher's data flow: the poll loop stops asking "did the text change" and starts asking "is this a turn boundary I have not analyzed".
- No API, storage, infra, auth, or external-service boundary changes. The provider call itself, its request shape, and its response contract are unchanged.

## 6. Requirements

- R1. Analysis is keyed to turn identity derived from the last user message in the session transcript, and that identity does not change while an agent produces output within a turn.
- R2. Within one turn a pane may produce at most two provider calls: one at turn start, when the newest transcript event is the user's own message, and one at turn end, when the pane leaves the working lifecycle state.
- R3. The persisted display state records which turn has been analyzed for each of the two phases, and the content fingerprint is gone from both the state schema and the trigger.
- R4. A pane's attention verdict does not change between turn boundaries as a result of transcript growth.
- R5. The existing safety backstops - the global inter-request spacer, the provider cooldown, and the daily request cap - remain in force, and a call deferred by any of them is still made once the deferral clears rather than being lost.
- R6. The interrupted-turn path and the abandoned-analysis path mark the current turn as analyzed so that neither re-asks the provider for the same turn.
- R7. A pane whose transcript contains no user message produces no provider call.

## 7. Acceptance Criteria

- AC1. While an agent produces output during a single turn, the turn identity computed from its transcript stays constant, and it changes when a new user message arrives.
- AC2. Driving one complete turn through the watcher - user prompt, streaming assistant output, then the pane leaving the working state - produces exactly two provider calls, and driving further output within that same turn produces none.
- AC3. The persisted display state no longer carries a content fingerprint, and it carries a record of the analyzed turn for the turn-start and turn-end phases independently.
- AC4. Given a turn whose analysis returned a question verdict, subsequent transcript growth within that same turn leaves the pane's displayed attention unchanged.
- AC5. A call suppressed by the global spacer, the provider cooldown, or the daily cap is still made on a later poll once that condition clears, and none of the three backstops is removed.
- AC6. An interrupted turn and an abandoned analysis both leave the pane in a state where the same turn is not analyzed again.
- AC7. A transcript with no user message produces no provider call.

## 8. PRD-Level Tasks

- T1. Introduce turn identity derived from the last user message, and prove it is stable across intra-turn transcript growth. Covers R1, AC1.
- T2. Replace the content fingerprint in the persisted display state with per-phase turn records, keeping the state file's existing corruption tolerance. Covers R3, AC3. Depends on: T1.
- T3. Rewrite the analysis trigger around the two turn boundaries, removing the content-hash comparison and preserving the existing working-state guard as the turn-start condition. Covers R2, R7, AC2, AC7. Depends on: T2.
- T4. Route the interrupted-turn and abandoned-analysis paths onto the new turn records. Covers R6, AC6. Depends on: T2.
- T5. Confirm the attention verdict is held for the duration of a turn and add regression coverage for the flip that motivated this change. Covers R4, AC4. Depends on: T3.
- T6. Confirm the deferral backstops still defer rather than drop, and keep all three in place. Covers R5, AC5. Depends on: T3.
- T7. Update the README and `docs/deployment.md` where they describe when the provider is called, so the documented trigger matches the implemented one. Covers R1, R2.

## 9. Verification Contract

### 9.1 Test Mode Contract

| Mode | Required For Done | Covers | Human Decision |
| --- | --- | --- | --- |
| build/static | yes | repo health, lint, formatting | none |
| automated behavior | yes | trigger firing, call counts, verdict stability, state schema | none |
| runtime observation | no | live call volume and flip count against the 2026-08-17 baseline | requires plugin reinstall on the live machine |
| live external API | no | real provider responses | would spend real budget; fakes cover the contract |

### 9.2 Required Agent Verification

| ID | Mode | Covers | Pass Intent | Required For Done | Can Be Blocked |
| --- | --- | --- | --- | --- | --- |
| V1 | build/static | R1-R7 | build, formatting, and lint do not regress across the trigger rewrite | yes | no |
| V2 | automated behavior | R1, AC1, SC1 | protects against the churn regression: a transcript growing with assistant output yields one unchanging turn identity, and a new user message yields a different one | yes | no |
| V3 | automated behavior | R2, R7, AC2, AC7, SC1, SC3 | protects against unbounded call volume: a counting fake provider records exactly two calls across one full turn, none for further intra-turn output, and none for a transcript with no user message | yes | no |
| V4 | automated behavior | R4, AC4, SC2 | protects against the verdict flip that motivated this work: once a question verdict lands, intra-turn transcript growth leaves the reported display and its attention symbol unchanged | yes | no |
| V5 | automated behavior | R3, AC3 | protects state compatibility: the persisted state round-trips both phase records, carries no fingerprint, and a previous-schema file still loads without stopping the watcher | yes | no |
| V6 | automated behavior | R5, AC5 | protects against dropped work: a call suppressed by a backstop is still made once the condition clears, and all three backstops remain present | yes | no |
| V7 | automated behavior | R6, AC6 | protects against re-asking a settled turn: after the interrupted or abandoned path runs, a later poll on the same turn issues no provider call | yes | no |
| V8 | runtime observation | R2, R4 | live call volume and attention-flip count on the reinstalled plugin improve against the 2026-08-17 baseline of 470 calls and 158 flips | no | yes |

### 9.3 Human Verification

- The sidebar's practical feel after the plugin is reinstalled on the live machine: whether the summary still appears fast enough when a turn starts, and whether holding a question symbol for a whole turn reads as stable rather than stale.
- Whether the retained backstops (D5) should now be deleted, which is a judgment about risk appetite rather than a testable property.

## 10. Risks And Open Decisions

- The running watcher is a separate plugin clone, so nothing here is observable on the live sidebar until Herdr reinstalls the plugin. The 2026-08-17 baseline of 470 calls and 158 flips cannot be compared against a new run inside this implementation.
- Holding the attention verdict for a whole turn means a verdict that is wrong at turn end stays wrong until the user acts. This is the intended trade against flapping, but it makes per-turn classification accuracy more visible than before.
- If Herdr's lifecycle flaps working to not-working repeatedly within one turn, the turn-end phase record is what prevents repeated calls. That record is therefore load-bearing in a way the old fingerprint was not.
- Turn identity by last-user-message hash collides if a user sends byte-identical consecutive prompts in one pane. The consequence is one skipped analysis, not a wrong verdict.

## 11. Implementation Guardrails

- Do not change the provider prompt, the model, the response schema, or the redaction path.
- Do not change hook classification, the display token set, the sort logic, or any color.
- Do not delete the global spacer, the provider cooldown, or the daily cap.
- Keep the state file's existing behavior of rebuilding from scratch when it cannot be parsed.
- Preserve the existing comment-carrying style of the codebase: explain why a rule exists where the reason is not evident from the code.

## 12. Implementation Result Report Contract

Report must state: the user-visible change to sidebar behavior; the trigger's before and after in one line each; the per-turn call bound actually enforced; conformance to the structure change in section 5; per-AC status; verification evidence by mode with the automated results embedded verbatim; deviations from this PRD with reasons; the assumptions from 4.3 that the user may still veto; the delivery mode used and, for local delivery, the exact state left in the working tree; and the fact that the live sidebar is unchanged until the plugin is reinstalled.
