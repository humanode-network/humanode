#!/bin/bash

CARGO_HACK_REPO_URL="https://github.com/taiki-e/cargo-hack"
CARGO_HACK_VERSION="v0.5.8"
CARGO_HACK_BASE_URL="${CARGO_HACK_REPO_URL}/releases/download/${CARGO_HACK_VERSION}"

install-cargo-hack() {
  local ARCHIVE_NAME="$1"
  local TO="$2"
  curl -sSL "${CARGO_HACK_BASE_URL}/${ARCHIVE_NAME}" | tar -xz -C "$TO"
}
