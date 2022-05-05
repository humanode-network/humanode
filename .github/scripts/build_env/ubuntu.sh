#!/bin/bash
set -euo pipefail

MOLD_GITHUB_REPO="rui314/mold"
MOLD_VERSION="1.2.1"
MOLD_INSTALL_PATH="${HOME}/.local"

mkdir -p "$MOLD_INSTALL_PATH"
wget \
  -O- \
  "https://github.com/$MOLD_GITHUB_REPO/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-$(uname -m)-linux.tar.gz" |
  tar -C "$MOLD_INSTALL_PATH" --strip-components=1 -xzf -

MOLD_PATH="${MOLD_INSTALL_PATH}/bin/mold"

"$MOLD_PATH" --version

cat >>./.cargo/config.toml <<EOF
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=${MOLD_PATH}"]
EOF

cat ./.cargo/config.toml
