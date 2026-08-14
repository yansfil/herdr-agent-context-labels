# Agent Context Labels for Herdr

[![Herdr](https://img.shields.io/badge/Herdr-%E2%89%A5%200.8.0-6c7086)](https://herdr.dev)
[![License](https://img.shields.io/github/license/yansfil/herdr-agent-context-labels)](LICENSE)

Compact task summaries and attention signals for Codex and Claude Code panes in [Herdr](https://herdr.dev).

The plugin keeps the Agents sidebar useful when several coding agents are running at once.
Each supported pane gets a short task label, lifecycle status, agent kind, and the time since its last state change.

```text
○  api-client       codex
   Retry policy cleanup       14s

?  docs             claude
   Choose installation path    2m
```

<p align="center">
  <a href="#install">install</a> ·
  <a href="#sidebar-layout">sidebar layout</a> ·
  <a href="#status-symbols">status symbols</a> ·
  <a href="#actions">actions</a> ·
  <a href="#privacy">privacy</a> ·
  <a href="#development">development</a>
</p>

## What it does

- Adds a maximum-30-character task summary to recognized Codex and Claude Code panes.
- Shows question, approval, error, working, unseen completion, idle, and unknown states as compact symbols.
- Alternates the working symbol between `○` and `●` without making the row disappear.
- Shows compact elapsed time such as `12s`, `4m`, `2h`, or `3d`.
- Keeps completion semantics aligned with Herdr's native `working`, `done`, `idle`, `blocked`, and `unknown` lifecycle states.
- Uses native agent hooks for high-confidence interaction signals and OpenRouter for task summaries and plain-text question detection.
- Publishes a `sort_rank` token and installs an `agent.view.set` sort on watcher start, so the sidebar orders panes attention-first: question/approval, then error, then unseen completion, then working, then idle.
- Runs one headless watcher, so no dedicated watcher pane is required.

The data flow is intentionally small:

```text
Herdr agent lifecycle ────────────────┐
Claude/Codex hooks ── attention ──────┼─> pane metadata tokens ─> Agents sidebar
local session JSONL ─> redact ─> LLM ─┘
```

Native hook state wins over semantic question detection, and both win over the ordinary Herdr lifecycle display.

## Status symbols

| Symbol | Meaning |
| --- | --- |
| `?` | The agent needs an answer from the user. |
| `!` | The agent is waiting for approval. |
| `×` | The agent stopped with an error. |
| `○` / `●` | The agent is working and the symbol is animating. |
| `●` | Background work finished and Herdr reports it as unseen. |
| `○` | The pane is idle and has been seen. |
| `~` | Herdr cannot classify the current lifecycle state confidently. |

Herdr owns the underlying lifecycle verdict.
In particular, `done` means background work finished before the tab was viewed, while `idle` means the settled pane has been seen.

## Requirements

- Herdr 0.8.0 or newer.
- macOS or Linux.
- Codex, Claude Code, or both.
- A stable Rust toolchain with Cargo for the current source-based installation.
- An OpenRouter API key for generated summaries and plain-text question detection.

Lifecycle symbols continue to work without OpenRouter.
If the key is missing or invalid, the watcher keeps existing summaries and records `credential_unavailable` locally without making an external request.

## Install

Install directly from GitHub:

```bash
herdr plugin install yansfil/herdr-agent-context-labels
```

Install Herdr's agent integrations for every agent runtime you use:

```bash
herdr integration install claude
herdr integration install codex
```

Set the OpenRouter credential in the environment used to start Herdr.
The startup script also reads a matching export from `~/.zshrc` when the Herdr server did not inherit the value:

```bash
export OPENROUTER_API_KEY='sk-or-v1-your-key-here'
```

The plugin startup hook runs when a Herdr server starts after restoring its session.
If the plugin was installed or linked while a server was already running, the watcher starts on the next Herdr server start because linking and enabling a plugin do not rerun startup hooks.

Confirm the registration:

```bash
herdr plugin list --plugin herdr-agent-context-labels
herdr plugin action list --plugin herdr-agent-context-labels
herdr integration status
```

## Sidebar layout

The plugin publishes metadata tokens, while the final row layout belongs to the user's Herdr configuration.
Add this to `~/.config/herdr/config.toml`:

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
    { token = "$status_idle", fg = "#a6adc8", bold = true },
    { token = "$status_stale", fg = "#6c7086", bold = true },
    "workspace",
    { token = "$agent_codex", fg = "#89b4fa", bold = true },
    { token = "$agent_claude", fg = "#fab387", bold = true },
  ],
  [
    { token = "$summary", fg = "#74c7ec", bold = true },
    { token = "$elapsed", fg = "#6c7086", dim = true },
  ],
]
```

The colors above fit a dark Catppuccin-style palette and can be changed independently from the plugin.

Validate and reload the configuration:

```bash
herdr config check
herdr server reload-config
```

## Agent hooks

Herdr's built-in Claude and Codex integrations provide the lifecycle states consumed by this plugin.
For precise `?`, `!`, and `×` classification, register the bundled `scripts/agent-hook.sh` as an additional hook in the agent runtime.

Use the plugin's absolute root path from `herdr plugin list --plugin herdr-agent-context-labels --json` and invoke:

```text
/bin/sh '<plugin-root>/scripts/agent-hook.sh'
```

Register that command for these events without replacing existing hooks:

| Event | Purpose |
| --- | --- |
| `PreToolUse` | Detect `AskUserQuestion` and `request_user_input`. |
| `PermissionRequest` | Distinguish approval from a direct question. |
| `PostToolUse` / `PostToolUseFailure` | Clear the matching pending tool state. |
| `StopFailure` | Report a stopped error. |
| `UserPromptSubmit` / `SessionStart` | Clear attention when the user or a new session resumes work. |

Tool-call IDs are matched before a completion clears pending attention, so an unrelated parallel tool cannot clear the wrong question or approval.
Without the optional hook wiring, summaries and Herdr lifecycle symbols still work, and OpenRouter can still identify direct plain-text questions.

## Actions

The plugin registers three explicit actions and does not force any keybinding:

```bash
herdr plugin action invoke refresh-active-pane-summary --plugin herdr-agent-context-labels
herdr plugin action invoke enable-automatic-summaries --plugin herdr-agent-context-labels
herdr plugin action invoke disable-automatic-summaries --plugin herdr-agent-context-labels
```

Enable and disable are separate idempotent operations.
Repeating either action leaves the same setting, which avoids an accidental double dispatch flipping the setting back.

An optional refresh keybinding looks like this:

```toml
[[keys.command]]
key = "prefix+r"
type = "plugin_action"
command = "herdr-agent-context-labels.refresh-active-pane-summary"
description = "refresh active pane summary"
```

## Summary generation

The watcher requests a new analysis after a supported agent settles and its sanitized session fingerprint changes.
Slow provider calls run outside the status loop, so lifecycle updates and the working animation continue while a request is in flight.

The fixed model is `poolside/laguna-s-2.1:free` and there is no fallback model.
The accepted provider output is exactly one JSON object with these fields:

```json
{"summary":"Retry policy cleanup","attention":"none"}
```

`summary` is normalized to one line and at most 30 characters, and the provider is instructed to write it in Korean.
`attention` accepts only `question` or `none` because approval and error states come from native hooks rather than model inference.

The local safety ceiling is 1,000 requests per UTC day, independent of the OpenRouter account's own limits.
Automatic summaries are enabled by default and the chosen setting survives watcher restarts.

## Privacy

The plugin reads the local JSONL session reported by Herdr for each supported pane.
Only the conversation from the latest user turn is considered, with an upper bound of 4,000 characters.

Before anything is sent to OpenRouter, the plugin:

- keeps only user and assistant prose;
- removes fenced code blocks;
- masks secret-looking values;
- masks email addresses;
- masks common absolute filesystem paths.

Raw prompts, raw model output, and credentials are not written to plugin logs.
Operational logs contain structured event names, pane IDs, agent kinds, stable failure classes, context length, and fingerprints rather than conversation text.

Local runtime data is stored under:

```text
~/.local/state/herdr-agent-context-labels/
```

The important files are:

| File | Contents |
| --- | --- |
| `display-state.json` | Last summaries, semantic attention, fingerprints, and lifecycle timestamps. |
| `hook-state.json` | Pending native-hook interaction state. |
| `settings.json` | Automatic-summary preference. |
| `usage.json` | Local UTC-day request count. |
| `events.jsonl` | Structured operational events and failure classes. |

The structured event log is restricted to the current user, and the remaining state files contain no credentials or raw conversation text.
Logs retain at most 30 days, three files, and 30 MiB per file rotation cycle.

## Troubleshooting

Check that Herdr recognizes the agents and that the plugin has published tokens:

```bash
herdr agent list
```

Inspect plugin command history and local structured events:

```bash
herdr plugin log list --plugin herdr-agent-context-labels
tail -n 50 ~/.local/state/herdr-agent-context-labels/events.jsonl
```

Verify one sanitized provider call without changing pane metadata:

```bash
./target/release/herdr-agent-context-labels verify-live-provider
```

Common event codes include `credential_unavailable`, `raw_session_unavailable`, `analysis_provider_failed`, and `analysis_skipped_daily_limit`.

## Development

The official Herdr plugin flow runs manifest build commands for GitHub installs, but `plugin link` does not build a local checkout.
Build first, then link the working directory:

```bash
git clone https://github.com/yansfil/herdr-agent-context-labels.git
cd herdr-agent-context-labels
cargo build --release --locked
herdr plugin link "$PWD"
```

Run the complete local checks:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

The runtime smoke test must run inside a Herdr-managed pane:

```bash
./scripts/verify-runtime.sh
```

## References

- [Herdr plugin documentation](https://herdr.dev/docs/plugins/)
- [herdr-reviewr](https://github.com/persiyanov/herdr-reviewr), whose README is a useful example of a clear install-first Herdr plugin guide

## License

[MIT](LICENSE)
