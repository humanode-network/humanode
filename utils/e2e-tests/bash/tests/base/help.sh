#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

# This test ensures we have an adequate and working help command and subcommands.
# It also serves as a snapshot test for the help texts, so we can track when
# the help texts change.

compare() {
  COMMAND="${@:-""}"
  FIXTURE="help."${COMMAND// /.}".stdout.txt"

  echo $FIXTURE

  OUTPUT="$("$HUMANODE_PEER_PATH" $COMMAND --help)"
  TEMPLATE="$(cat "$SCRIPT_DIR/../../fixtures/$FIXTURE")"

  DIFF_CMD_ARGS=(
    -u
    -b
    -I '^humanode-peer {{sha}}$'        # template
    -I '^humanode-peer [0-9a-f]\{40\}$' # expected
  )

  if ! DIFF="$(diff "${DIFF_CMD_ARGS[@]}" <(printf '%s' "$TEMPLATE") <(printf '%s' "$OUTPUT"))"; then
    printf "Output did not match:\n%s\n" "$DIFF"
    exit 1
  fi
}

COMMANDS=(
  "key"
  "key generate-node-key"
  "key generate"
  "key inspect"
  "key inspect-node-key"
  "key insert"
)

for COMMAND in "${COMMANDS[@]}"; do
  compare $COMMAND
done

printf "Test succeded\n" >&2
