#!/bin/bash
set -euo pipefail

brew install \
  coreutils

.github/scripts/protoc.sh
