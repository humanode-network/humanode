#!/bin/bash
set -uo pipefail

# Set up commands.
COMMAND_TO_RUN=(target/debug/humanode-peer --dev)

# Set up timeout.
TIME_IN_SEC=60

run_with_timeout() {
    ANCHOR=$SECONDS
    # shellcheck disable=SC2251
    timeout "$@"
    EXITCODE="$?"
    if [[ "$EXITCODE" -ne 124 ]]; then
        printf "App run lasted for %d seconds and was terminated with error code %d\n" "$(( "$SECONDS" - "$ANCHOR" ))" "$EXITCODE" >&2
        exit
    fi
    printf "App run lasted for %d seconds and was terminated by timeout\n" "$TIME_IN_SEC" >&2
}

# Run with empty state, then 2nd time with non-empty state.
for (( i=1; i <= 2; i++ )); do
    run_with_timeout "$TIME_IN_SEC" "${COMMAND_TO_RUN[@]}"
done
