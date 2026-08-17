---
topic: docs-single-source
status: complete
---

## Goal

The project ships two overlapping install guides, `README.md` in English and `INSTALL.md` in Korean, with no declared authority between them. They have drifted, and a reader who follows either one ends up somewhere different: README's sidebar example colors only 7 of the 11 status tokens the plugin publishes, so the unread error, question, and approval states render with no color at all; INSTALL.md gives `status_working` and `status_done` the same green even though they share the `●` symbol, making working and finished indistinguishable; and INSTALL.md names a free nvidia model and tells the reader there is no billing, while the code calls `openai/gpt-5.6-luna`.

This run makes `README.md` the single canonical document, reduces `INSTALL.md` to a Korean quick start that defers to it, corrects every factual error found in the review, documents the surfaces that were missing entirely, and adds a test that fails when a documented config example drifts from the tokens the code actually publishes.

## Non-goals

- Changing any runtime behavior, token name, color the plugin publishes, or default sort order. This is a documentation run; the only code added is a test that reads the docs.
- Translating README in full. `INSTALL.md` stays a Korean quick start rather than becoming a parallel full document, because a second full document is the condition that produced this drift.
- Editing the user's live `~/.config/herdr/config.toml`. The repository documents what to write; the machine's own config is the user's.

## Checks

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

## Acceptance Criteria

- AC1. `README.md` is stated to be the canonical reference for installation and configuration, and `INSTALL.md` points to it rather than restating the full procedure, so the two documents can no longer disagree on a detail.
- AC2. Every status token the plugin publishes is present in the documented sidebar configuration example, and `status_working` and `status_done` are given different colors there, because they render the same symbol.
  - check: `cargo test`
- AC3. No document claims the analysis model is free or names a model other than the one the code calls, and the model named in the docs is kept in agreement with the code by a test rather than by memory.
  - check: `cargo test`
- AC4. The privacy description states that the analysis context spans the last two user turns, matching `analysis_context`, instead of only the latest turn.
- AC5. `README.md` documents the optional `sort-order.json` override: where the file lives, what it does to the built-in order, that a partial list keeps the remaining states in their default relative order, and that a watcher restart is needed. The default order shown is the current one, with the failed turn first.
- AC6. The documentation explains what makes a pane unread rather than seen, and that the `_new` token variants are the unread form of question, approval, and error, so a reader can tell why one `?` is colored differently from another.
- AC7. The documentation states that an approval symbol also arrives from Herdr's own `blocked` lifecycle without any hook wiring, and that the error symbol is the one signal that cannot appear at all until the bundled hook is registered.
- AC8. The troubleshooting guidance no longer instructs the reader to edit a `analysis_fingerprint` field that the code no longer has, and the `analyze-stdin` subcommand is documented alongside `verify-live-provider`.
