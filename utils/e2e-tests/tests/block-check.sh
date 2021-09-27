#!/bin/bash
set -euo pipefail

get_address() {
    read -ra array <<< "$(target/debug/humanode-peer key inspect "$@" | grep "SS58 Address:")"
    echo "${array[2]}"
}

spawn_node_for_20_sec() {
    target/debug/humanode-peer --dev &
    sleep 20
    kill %1
}

# Run the node.
spawn_node_for_20_sec &

# Get the address.
ADDR="$(get_address "//Alice")"

# Send TX and wait for block creation.
# The test will also fail if no block is created within 20 sec.
POLKA_JSON="$(timeout 20 polkadot-js-api --ws "ws://127.0.0.1:9944" --seed "//Alice" tx.balances.transfer "$ADDR" 10000)"

# Log polkadot-js-api response.
printf "polkadot-js-api response:\n%s" "$POLKA_JSON" >&2

# Look for "InBlock" field in response.
echo "$POLKA_JSON" | grep "InBlock"

# Wait for spawned node to die, so the next test won't face the LOCK.
sleep 20
