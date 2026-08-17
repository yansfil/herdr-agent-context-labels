# Deployment

## The running watcher is not this checkout

Herdr installs the plugin as its own git clone, and that clone is what actually runs:

```text
~/.config/herdr/plugins/github/herdr-agent-context-labels-<hash>/
```

Building this repository changes nothing about the live sidebar.
Confirm which binary is running before concluding a fix did or did not work:

```sh
ps -eo pid,etime,command | grep 'herdr-agent-context-labels watch' | grep -v grep
```

The path in that output is the answer.

## Getting a change into the running watcher

The supported route is to push and let Herdr reinstall the plugin, which reruns the build script.

For a local trial before pushing, overwrite the installed binary:

```sh
P=~/.config/herdr/plugins/github/herdr-agent-context-labels-<hash>
cargo build --release
cp target/release/herdr-agent-context-labels "$P/target/release/herdr-agent-context-labels"
```

This is temporary. A plugin rebuild or reinstall discards it, so a trial that works this way is not yet delivered.

## Restarting the watcher

The startup wrapper launches the watcher once and waits on the Herdr server; it does not respawn a watcher that exits.
Killing the watcher leaves no watcher running, and the sidebar silently stops updating.

Restart it by hand, or restart the Herdr server to get the wrapper to run the normal startup path:

```sh
kill <watcher-pid>
nohup /bin/sh "$P/scripts/start-watcher.sh" >/tmp/watcher.log 2>&1 &
```

The startup script reads `OPENROUTER_API_KEY` from the environment, falling back to an `export` line in `~/.zshrc`, because a Herdr server started outside a login shell does not inherit it.

## State lives outside both checkouts

```text
~/.local/state/herdr-agent-context-labels/
├── events.jsonl        # append-only log; the first place to look
├── display-state.json  # per-pane summary, verdict, analyzed turn per phase
├── usage.json          # {"day": <unix days>, "requests": N} against DAILY_REQUEST_LIMIT
└── settings.json       # automatic-summaries toggle
```

`events.jsonl` is the diagnostic record.
Counting its events by day and by `detail` is what identifies a provider problem:

```sh
grep '<pane-id>' ~/.local/state/herdr-agent-context-labels/events.jsonl | tail -30
```

`usage.json` is a local counter, not a provider quota.
Resetting `requests` to 0 restores analysis for the current day, which is the recovery step after the budget was spent on failures rather than on answers.

## Sidebar colors are the user's, not the plugin's

The plugin publishes `status_*` tokens; `~/.config/herdr/config.toml` decides how they are painted.
A status change that depends on being visually distinct needs a matching config edit, followed by:

```sh
herdr config check
herdr server reload-config
```

`status_working` and `status_done` render the same `●` symbol, so they must be given different colors.
