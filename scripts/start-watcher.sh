#!/bin/sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
export PATH="$HOME/.local/bin:$HOME/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

# Herdr plugin startup can inherit an environment that predates the user's
# exported credential. Read only this plugin's registered value instead of
# executing the interactive shell profile.
if [ -z "${OPENROUTER_API_KEY:-}" ] && [ -r "$HOME/.zshrc" ]; then
  openrouter_key="$(/usr/bin/awk '
    /^[[:space:]]*export[[:space:]]+OPENROUTER_API_KEY=/ {
      value = $0
      sub(/^[[:space:]]*export[[:space:]]+OPENROUTER_API_KEY=/, "", value)
      gsub(/^[[:space:]\047\042]+|[[:space:]\047\042]+$/, "", value)
    }
    END { printf "%s", value }
  ' "$HOME/.zshrc")"
  if [ -n "$openrouter_key" ]; then
    export OPENROUTER_API_KEY="$openrouter_key"
  fi
  unset openrouter_key
fi

exec "$root/target/release/herdr-agent-context-labels" watch
