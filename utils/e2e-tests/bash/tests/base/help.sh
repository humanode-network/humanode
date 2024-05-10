#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

# This test ensures we have an adequate and working help command and subcommands.
# It also serves as a snapshot test for the help texts, so we can track when
# the help texts change.

compare() {
  SUBCOMMAND="${@:-""}"

  # Replace subcommands spaces by dots to prepare fixture filename.
  FIXTURE_FILENAME="help."${SUBCOMMAND// /.}".stdout.txt"
  # Avoid having double dots.
  FIXTURE_FILENAME=${FIXTURE_FILENAME//../.}

  OUTPUT="$("$HUMANODE_PEER_PATH" $SUBCOMMAND --help)"
  TEMPLATE="$(cat "$SCRIPT_DIR/../../fixtures/help-output/$FIXTURE_FILENAME")"

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

SUBCOMMANDS=(
  ""
  "key"
  "key generate-node-key"
  "key generate"
  "key inspect"
  "key inspect-node-key"
  "key insert"
  "build-spec"
  "check-block"
  "export-blocks"
  "export-state"
  "import-blocks"
  "purge-chain"
  "revert"
  "benchmark"
  "benchmark pallet"
  "benchmark storage"
  "benchmark overhead"
  "benchmark block"
  "benchmark machine"
  "benchmark extrinsic"
  "frontier-db"
  "export-embedded-runtime"
  "try-runtime"
)

for SUBCOMMAND in "${SUBCOMMANDS[@]}"; do
  compare $SUBCOMMAND
done

printf "Test succeded\n" >&2
