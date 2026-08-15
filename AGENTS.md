# Agent Notes

## Project notes

- The running watcher is Herdr's own plugin clone, not this checkout, and its startup wrapper does not respawn it: [docs/deployment.md](docs/deployment.md).

<!-- harness:agents-namespace:start -->
## Harness Namespace (`agents/`)

This project uses the engineering-harness PRD pipeline. Agent-facing assets live in one visible namespace:

- `agents/prd/` - PRD contracts, committed and human-approved before implementation.
- `agents/rules/` - learned rules: `INDEX.md` is the ledger, `invariants/` hold machine-checked rules (trigger globs + executable check) that gate delivery, `pending/` holds lessons that have not landed yet.
- `agents/implement/` - runtime state and evidence, gitignored (policy: one line `agents/implement/`), never hand-edited.
- `agents/config.json` - pipeline configuration, committed.

Conventions:

- AGENTS.md is the main agent context file; CLAUDE.md is always a symlink to it.
- Before planning work that touches files matched by a rule trigger, consult `node ~/.claude/skills/implement/scripts/prd_state_harness.js rules relevant --paths <files>` or read `agents/rules/INDEX.md`.
- Rules are added through `rules add` (never hand-edit the ledger); every rule cites evidence from a real run or incident.
<!-- harness:agents-namespace:end -->
