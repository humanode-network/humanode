#!/bin/bash
set -euo pipefail

# Run try-runtime on-runtime-upgrade command.
"$HUMANODE_PEER_PATH" try-runtime \
  --detailed-log-output \
  --runtime "$RUNTIME_WASM"  \
  on-runtime-upgrade \
  live \
  --uri "$LIVE_NETWORK_URL"

printf "Test succeded\n" >&2
