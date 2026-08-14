#!/bin/sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
exec "$root/target/release/herdr-agent-context-labels" set-automatic-summaries --enabled true
