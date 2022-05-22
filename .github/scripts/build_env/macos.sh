#!/bin/bash
set -euo pipefail

# Undo the Rosetta hack if needed.
PROC_TRANSLATED="$(sysctl -n sysctl.proc_translated || true)"
CURRENT_ARCH="$(uname -m)"
if [[ "$PROC_TRANSLATED" == "1" && "$CURRENT_ARCH" == "x86_64" ]]; then
  if [[ "${MACOS_ROSETTA_HACK_UNDONE:-}" == "true" ]]; then
    printf "Seems like we've fallen into a loop of rosetta hack undoing, crashing the run.\n" >&2
    exit 1
  fi

  export MACOS_ROSETTA_HACK_UNDONE=true
  # Here we assume the native arch is arm64...
  set -x
  exec arch -arm64 "$0" "$@"
fi

brew install \
  coreutils

if [[ "$(uname -m)" == "x86_64" ]]; then
  echo "Intel-specific setup..."

  brew install \
    michaeleisel/zld/zld

  ZLD_PATH="$(which zld)"

  "$ZLD_PATH" -v

  cat >>./.cargo/config.toml <<EOF
[target.x86_64-apple-darwin]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=${ZLD_PATH}"]
EOF

  cat ./.cargo/config.toml
fi
