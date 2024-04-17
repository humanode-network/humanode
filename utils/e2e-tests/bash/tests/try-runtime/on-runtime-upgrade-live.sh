#!/bin/bash
set -euo pipefail

LIVE_NETWORK_URL="wss://explorer-rpc-ws.mainnet.stages.humanode.io:443"

# Run try-runtime on-runtime-upgrade command.
"$HUMANODE_PEER_PATH" try-runtime \
  --detailed-log-output \
  --runtime "$RUNTIME_WASM_PATH"  \
  on-runtime-upgrade \
  --checks=all \
  live \
  --uri "$LIVE_NETWORK_URL"

printf "Test succeded\n" >&2
