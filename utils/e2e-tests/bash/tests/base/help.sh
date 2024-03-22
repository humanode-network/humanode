#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

# This test ensures we have an adequate and working help command.
# It also serves as a snapshot test for the help text, so we can track when
# the help text changes.

OUTPUT="$("$@" --help)"
TEMPLATE="$(cat "$SCRIPT_DIR/../../fixtures/help.stdout.txt")"

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

printf "Test succeded\n" >&2
