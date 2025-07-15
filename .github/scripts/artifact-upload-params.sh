#!/usr/bin/env bash
set -euo pipefail

if [[ "$ARTIFACT_SELECTOR" == 'runtime' ]]; then
  ARTIFACT_PATH='target/release/wbuild/humanode-runtime/humanode_runtime.compact.compressed.wasm'
  ARTIFACT_NAME='humanode-runtime'

  if [[ -n "$MODE_ARTIFACT_MARKER" ]]; then
    ARTIFACT_NAME+="-$MODE_ARTIFACT_MARKER"
  fi

  ARTIFACT_NAME+='.wasm'
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
