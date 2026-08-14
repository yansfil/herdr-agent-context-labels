#!/bin/sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
bin="$root/target/release/herdr-agent-context-labels"

if [ -x "$bin" ]; then
  exit 0
fi

cd "$root"
cargo build --release --locked
