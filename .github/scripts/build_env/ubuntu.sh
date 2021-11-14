#!/bin/bash
set -euo pipefail

source .github/scripts/build_env/lib/common.sh

install-cargo-hack "cargo-hack-x86_64-unknown-linux-gnu.tar.gz" /usr/local/bin
chmod +x /usr/local/bin/cargo-hack
