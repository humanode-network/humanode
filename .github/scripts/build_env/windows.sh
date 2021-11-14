#!/bin/bash
set -euo pipefail

source .github/scripts/build_env/lib/common.sh

# Prepare the bin dir.
BIN_DIR="C:/bin"
mkdir -p "$BIN_DIR"
echo "$BIN_DIR" >>"$GITHUB_PATH"
export PATH="$BIN_DIR:$PATH"

install-cargo-hack "cargo-hack-x86_64-pc-windows-msvc.tar.gz" "$BIN_DIR"
