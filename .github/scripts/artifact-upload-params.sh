#!/usr/bin/env bash
set -euo pipefail

if [[ "$ARTIFACT_SELECTOR" == 'runtime' ]]; then
  if [[ -z "$MODE_ARTIFACT_MARKER" ]]; then
    printf 'MODE_ARTIFACT_MARKER must not be empty\n' >&2
    exit 1
  fi

  ARTIFACT_PATH='target/release/wbuild/humanode-runtime/humanode_runtime.compact.compressed.wasm'
  ARTIFACT_NAME="humanode-runtime-${MODE_ARTIFACT_MARKER}.wasm"
else
  ARTIFACT_PATH='target/release/humanode-peer'
  ARTIFACT_NAME="humanode-peer-$(rustc -vV | sed -n 's|host: ||p')"

  if [[ "$PLATFORM_ARTIFACT_MARKER" != "" ]]; then
    ARTIFACT_NAME="${ARTIFACT_NAME}-${PLATFORM_ARTIFACT_MARKER}"
  fi

  if [[ "${PATHEXT:-""}" != "" ]]; then
    ARTIFACT_PATH="${ARTIFACT_PATH}.exe"
  fi
fi

printf 'artifact-path=%s\n' "$ARTIFACT_PATH" >> "$GITHUB_OUTPUT"
printf 'artifact-name=%s\n' "$ARTIFACT_NAME" >> "$GITHUB_OUTPUT"

printf 'Packaged `%s` into `%s`.\n' \
  "$ARTIFACT_PATH" \
  "$ARTIFACT_NAME" \
  >> "$GITHUB_STEP_SUMMARY"
