#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

# This test ensures we have an adequate and working help command and subcommands.
# It also serves as a snapshot test for the help texts, so we can track when
# the help texts change.

assert_matches_snapshot() {
  local EXPECTED_PATH="$1"
  local ACTUAL_PATH="$2"

  DIFF_CMD_ARGS=(
    -u
    -b
    -I '^humanode-peer {{sha}}$'        # template
    -I '^humanode-peer [0-9a-f]\{40\}$' # expected
  )

  if ! DIFF="$(diff "${DIFF_CMD_ARGS[@]}" "$EXPECTED_PATH" "$ACTUAL_PATH")"; then
    printf "Output did not match:\n%s\n" "$DIFF"
    exit 1
  fi
}

compare() {
  local COMMAND="${@:-""}"

  # Replace commands spaces by dots to prepare fixture filename.
  FIXTURE_FILENAME="help."${COMMAND// /.}".stdout.txt"
  # Avoid having double dots.
  FIXTURE_FILENAME=${FIXTURE_FILENAME//../.}

  OUTPUT="$("$HUMANODE_PEER_PATH" $COMMAND --help)"
  TEMPLATE="$(cat "$SCRIPT_DIR/../../fixtures/help-output/$FIXTURE_FILENAME")"

  assert_matches_snapshot <(printf '%s' "$TEMPLATE") <(printf '%s' "$OUTPUT")

  local SUBCOMMANDS=($(awk '/Commands:/,/Options:/{if(/Commands:|Options:/) next; print}' <(printf '%s' "$OUTPUT") | awk '{if ($1) print $1}'))

  if [[ "${SUBCOMMANDS-}" ]]; then
    for SUBCOMMAND in "${SUBCOMMANDS[@]}"; do
      if [[ $SUBCOMMAND != "help" ]]; then
        compare "$COMMAND $SUBCOMMAND"
      fi
    done
  fi
}

compare

printf "Test succeded\n" >&2
