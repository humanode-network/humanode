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

extract_subcommands() {
  local OUTPUT_PATH="$1"

  awk '!NF{f=0} /Commands:/ {f=1; next} { if (f) print $1;}' "$OUTPUT_PATH"
}

read_into_array() {
  ARRAY=()
  while IFS= read -r ARRAY_ITEM; do
    ARRAY+=("${ARRAY_ITEM}")
  done
}

compare() {
  local COMMAND=("$@")

  local SNAPSHOT_FILENAME
  local FULL_INVOCATION

  if [[ "${#COMMAND[@]}" -eq 0 ]]; then
    SNAPSHOT_FILENAME="help.stdout.txt"
    FULL_INVOCATION=("$HUMANODE_PEER_PATH" --help)
  else
    local FIXTURE_FILE_PART

    FIXTURE_FILE_PART="$(
      IFS="."
      echo "${COMMAND[*]}"
      IFS=" "
    )"
    SNAPSHOT_FILENAME="help.${FIXTURE_FILE_PART}.stdout.txt"
    FULL_INVOCATION=("$HUMANODE_PEER_PATH" "${COMMAND[@]}" --help)
  fi

  printf "Checking \"%s\" against \"%s\"\n" "${FULL_INVOCATION[*]}" "$SNAPSHOT_FILENAME"

  OUTPUT="$("${FULL_INVOCATION[@]}")"
  TEMPLATE="$(cat "$SCRIPT_DIR/../../fixtures/help-output/$SNAPSHOT_FILENAME")"

  assert_matches_snapshot <(printf '%s' "$TEMPLATE") <(printf '%s' "$OUTPUT")

  read_into_array < <(extract_subcommands <(printf '%s' "$OUTPUT"))
  if [[ "${#ARRAY[@]}" -eq 0 ]]; then
    # No subcommands, do not recurse.
    return
  fi
  local SUBCOMMANDS=("${ARRAY[@]}")

  printf "Detected subcommands:\n"
  printf -- "- %s\n" "${SUBCOMMANDS[@]}"

  for SUBCOMMAND in "${SUBCOMMANDS[@]}"; do
    if [[ $SUBCOMMAND == "help" ]]; then
      continue
    fi

    IFS=" " read -r -a SUBCOMMAND_ARRAY <<<"$SUBCOMMAND"

    local FULL_SUBCOMMAND=()
    if [[ "${#COMMAND[@]}" -ne 0 ]]; then
      FULL_SUBCOMMAND+=("${COMMAND[@]}")
    fi
    FULL_SUBCOMMAND+=("${SUBCOMMAND_ARRAY[@]}")

    compare "${FULL_SUBCOMMAND[@]}"
  done
}

compare

printf "Test succeded\n" >&2
