#!/bin/bash
set -euo pipefail

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

.github/scripts/protoc.sh
