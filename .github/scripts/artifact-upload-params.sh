#!/usr/bin/env bash
set -euo pipefail

RUNTIME_PATH='target/release/wbuild/humanode-runtime/humanode_runtime.compact.compressed.wasm'

case "$ARTIFACT_SELECTOR" in
  peer)
    ARTIFACT_PATH='target/release/humanode-peer'
    ARTIFACT_NAME="humanode-peer-$(rustc -vV | sed -n 's|host: ||p')"

    if [[ "$PLATFORM_ARTIFACT_MARKER" != "" ]]; then
      ARTIFACT_NAME="${ARTIFACT_NAME}-${PLATFORM_ARTIFACT_MARKER}"
    fi

    if [[ "${PATHEXT:-""}" != "" ]]; then
      ARTIFACT_PATH="${ARTIFACT_PATH}.exe"
    fi
    ;;

  runtime)
    ARTIFACT_PATH="$RUNTIME_PATH"
    ARTIFACT_NAME='humanode-runtime.wasm'
    ;;

  runtime-evm-tracing)
    ARTIFACT_PATH="$RUNTIME_PATH"
    ARTIFACT_NAME='humanode-runtime-evm-tracing.wasm'
    ;;

  *)
    printf 'Unknown artifact selector `%s`\n' "$ARTIFACT_SELECTOR" >&2
    exit 1
    ;;
esac

printf 'artifact-path=%s\n' "$ARTIFACT_PATH" >> "$GITHUB_OUTPUT"
printf 'artifact-name=%s\n' "$ARTIFACT_NAME" >> "$GITHUB_OUTPUT"

printf 'Packaged `%s` into `%s`.\n' \
  "$ARTIFACT_PATH" \
  "$ARTIFACT_NAME" \
  >> "$GITHUB_STEP_SUMMARY"
