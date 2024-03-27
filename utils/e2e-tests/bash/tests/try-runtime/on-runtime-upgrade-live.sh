#!/bin/bash
set -euo pipefail

HUMANODE_PEER_PATH="${1?Provide the path to the humanode peer as the first argument}"
shift

RUNTIME_WASM="${1?Provide the path to runtime wasm file as the second argument}"

# Run try-runtime on-runtime-upgrade command.
"$HUMANODE_PEER_PATH" try-runtime \
  --detailed-log-output \
  --runtime "$RUNTIME_WASM"  \
  on-runtime-upgrade \
  live \
  --uri "wss://explorer-rpc-ws.it6.stages.humanode.io:443"

printf "Test succeded\n" >&2
