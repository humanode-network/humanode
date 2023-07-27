#!/bin/bash
set -euo pipefail

get_address() {
  "$COMMAND" key inspect "$@" | grep "SS58 Address:" | awk '{print $3}'
}

# Set up command.
COMMAND="$1"

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"; pkill -P "$$"' EXIT

# Run the node.
"$COMMAND" --dev --base-path "$TEMPDIR" &

# Get the address.
ADDR="$(get_address "//Alice")"

# Send TX and wait for block creation.
# The test will also fail if no block is created within 20 sec.
POLKA_JSON="$(timeout 20 yarn --silent polkadot-js-api --ws "ws://127.0.0.1:9944" --seed "//Alice" tx.balances.transfer "$ADDR" 10000)"

# Log polkadot-js-api response.
printf "polkadot-js-api response:\n%s\n" "$POLKA_JSON" >&2

# Look for a status update with "inBlock" status. Fail the test if absent.
jq \
  --slurp \
  --exit-status \
  '.[] | select(.transfer.status.inBlock != null) | length == 1' <<<"$POLKA_JSON"

printf "Test succeded\n" >&2
