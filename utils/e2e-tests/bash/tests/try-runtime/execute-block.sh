#!/bin/bash
set -euo pipefail

HUMANODE_PEER_PATH="${1?Provide the path to the humanode peer as the first argument}"

# A helper function to keep the node running until a requested block number is imported.
wait_block_with_timeout() {
  REQUESTED_BLOCK_NUMBER="$1"
  TIMEOUT="$2"
  while true; do
    # Sleep 6 secs as it's an approximate time to produce a block.
    sleep 6
    # Obtain the requested block hash.
    BLOCK_HASH_JSON="$(yarn workspace humanode-e2e-tests-bash polkadot-js-api --ws "ws://127.0.0.1:9944" rpc.chain.getBlockHash "$REQUESTED_BLOCK_NUMBER")"
    # Check if the hash is not null.
    if [[ $(grep -L "0x0000000000000000000000000000000000000000000000000000000000000000" <<<"$BLOCK_HASH_JSON") ]]; then
      break
    fi
    if [[ "$SECONDS" -gt "$TIMEOUT" ]]; then
      printf "Terminated by timeout" >&2
      exit
    fi
  done
}

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"; pkill -P "$$"' EXIT

# Run the node.
"$HUMANODE_PEER_PATH" --dev --base-path "$TEMPDIR" &

# Keep the node running until 5th block is imported.
wait_block_with_timeout 5 50

# Run try-runtime execute-block command.
"$HUMANODE_PEER_PATH" try-runtime --runtime existing --detailed-log-output execute-block live --uri "ws://127.0.0.1:9944"

printf "Test succeded\n" >&2
