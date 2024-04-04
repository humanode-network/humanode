#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
cd "$SCRIPT_DIR/../.."

ZOMBIENET_EXEC_PATH="$(realpath -s "$ZOMBIENET_EXEC_PATH")"

HUMANODE_PEER_PATH="$(realpath -s "$HUMANODE_PEER_PATH")"
export HUMANODE_PEER_PATH

ZOMBIENET_PLAYGROUND_PATH="./zombienet-plaground"
export ZOMBIENET_PLAYGROUND_PATH

"$ZOMBIENET_EXEC_PATH" spawn -p native ./zombienet/block-building/block-building.toml -d "$ZOMBIENET_PLAYGROUND_PATH"
