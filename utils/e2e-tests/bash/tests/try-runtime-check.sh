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
while true; do
  sleep 6
  echo "Trying..."
  BLOCK_HASH_JSON="$(yarn polkadot-js-api --ws "ws://127.0.0.1:9944" rpc.chain.getBlockHash 5)"
  if [[ $(grep -L "0x0000000000000000000000000000000000000000000000000000000000000000" <<<"$BLOCK_HASH_JSON") ]]; then
    break
  fi
done

# Run try-runtime execute-block command.
"$COMMAND" try-runtime --runtime existing execute-block live --uri "ws://127.0.0.1:9944"

printf "Test succeded\n" >&2
