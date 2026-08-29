#!/bin/bash
set -euo pipefail

get_address() {
  "$HUMANODE_PEER_PATH" key inspect "$@" | grep "SS58 Address:" | awk '{print $3}'
}

RPC_URL_HTTP="http://127.0.0.1:9944"
RPC_URL_WS="ws://127.0.0.1:9944"

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"; pkill -P "$$"' EXIT

# Run the node.
"$HUMANODE_PEER_PATH" --dev --base-path "$TEMPDIR" &

# Get the address.
ADDR="$(get_address "//Alice")"

# Wait for the RPC endpoint to come up before submitting anything, so that the
# client does not race the node startup. Fail explicitly if it never does.
RPC_WAIT_TIMEOUT_SEC=30
RPC_WAIT_ANCHOR="$SECONDS"
until curl \
  --silent \
  --output /dev/null \
  --header 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
  "$RPC_URL_HTTP"; do
  if (("$SECONDS" - "$RPC_WAIT_ANCHOR" >= RPC_WAIT_TIMEOUT_SEC)); then
    printf "RPC endpoint did not come up within %d seconds\n" "$RPC_WAIT_TIMEOUT_SEC" >&2
    exit 1
  fi
  sleep 0.2
done
printf "RPC endpoint is up after %d seconds\n" "$(("$SECONDS" - "$RPC_WAIT_ANCHOR"))" >&2

# Send TX and wait for block creation.
# The test will also fail if no block is created within 30 sec.
#
# The client output is streamed to stderr as it arrives (in addition to being
# captured), so that it is visible in the logs even if the command times out.
# DEBUG=api-ws makes the polkadot-js WebSocket provider log every request,
# response and connection event to stderr; DEBUG_MAX truncates each logged
# value (the metadata response alone is hundreds of KB otherwise).
POLKA_JSON="$(
  DEBUG=api-ws DEBUG_MAX=1000 timeout 30 \
    yarn workspace humanode-e2e-tests-bash polkadot-js-api \
    --ws "$RPC_URL_WS" \
    --seed "//Alice" \
    tx.balances.transfer "$ADDR" 10000 |
    tee /dev/stderr
)"

# Log polkadot-js-api response.
printf "polkadot-js-api response:\n%s\n" "$POLKA_JSON" >&2

# Look for a status update with "inBlock" status. Fail the test if absent.
jq \
  --slurp \
  --exit-status \
  '.[] | select(.transfer.status.inBlock != null) | length == 1' <<<"$POLKA_JSON"

printf "Test succeeded\n" >&2
