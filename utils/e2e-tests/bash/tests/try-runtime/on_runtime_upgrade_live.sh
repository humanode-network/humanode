#!/bin/bash
set -euo pipefail

# Set up command.
COMMAND="$1"

# Run try-runtime on-runtime-upgrade command.
"$COMMAND" try-runtime \
  --detailed-log-output \
  --runtime ./target/release/wbuild/humanode-runtime/humanode_runtime.wasm  \
  on-runtime-upgrade \
  live \
  --uri "wss://explorer-rpc-ws.it6.stages.humanode.io:443"

printf "Test succeded\n" >&2
