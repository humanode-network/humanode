#!/bin/bash
set -euo pipefail

# Set up command.
COMMAND="$1"

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"; pkill -P "$$"' EXIT

# Run the node.
"$COMMAND" --dev --base-path "$TEMPDIR" &

# Kepp the node running to have around 3 finalized blocks.
sleep 40

# Run try-runtime execute-block command.
"$COMMAND" try-runtime --runtime existing execute-block live --uri "ws://127.0.0.1:9944"

printf "Test succeded\n" >&2
