#!/bin/bash
set -euo pipefail

SRC="$1"
shift

for DST in "$@"; do
  docker tag "$SRC" "$DST"
done

for TAG in "$@"; do
  docker push "$TAG"
done
