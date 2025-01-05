#!/bin/bash
set -euo pipefail

source depversions.sh

PROTOBUF_VERSION="${PROTOC_VERSION:?Check depversions.sh}"
PROTOBUF_RELEASES_URL="https://github.com/protocolbuffers/protobuf/releases"

# Aw, how cure, they use python instead of uname...
PROTOBUF_PLATFORM="$(python3 -c 'import sysconfig; print(sysconfig.get_platform())')"
case "$PROTOBUF_PLATFORM" in
"macosx-"*"-arm64")
  PROTOBUF_PLATFORM="osx-aarch_64"
  ;;
"macosx-"*"-x86_64")
  PROTOBUF_PLATFORM="osx-x86_64"
  ;;
"macosx-"*"-universal"*)
  PROTOBUF_PLATFORM="osx-universal_binary"
  ;;
"linux-aarch64")
  PROTOBUF_PLATFORM="linux-aarch_64"
  ;;
esac

URL="$PROTOBUF_RELEASES_URL/download/v$PROTOBUF_VERSION/protoc-$PROTOBUF_VERSION-$PROTOBUF_PLATFORM.zip"

printf "Downloading protobuf from %s\n" "$URL"
curl -sSL "$URL" -o protoc.zip

INSTALL_PATH="/usr/local"
sudo unzip -o -d "$INSTALL_PATH" protoc.zip
rm protoc.zip

protoc --version
