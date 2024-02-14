#!/bin/bash
set -euo pipefail

# Change directory to the project root.
cd "$(dirname "${BASH_SOURCE[0]}")"/..

source depversions.sh

# Build parameters.
PLATFORM="linux/amd64"
BUILDER_CONTAINER_BASE="rust:bookworm"
RUNTIME_CONTAINER_BASE="debian:bookworm"
BUILDER_CONTAINER_TAG="humanode-builder"
RUNTIME_CONTAINER_TAG="humanode"
BUILD_VOLUMES_PATH="$(pwd)/target/docker/$PLATFORM"
HOST_TARGET="${BUILD_VOLUMES_PATH}/target"

# Enable BuildKit.
export DOCKER_BUILDKIT=1

# Prepare the docker context for the build environment.
TMP_BUILDER_DATA_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_BUILDER_DATA_DIR"' EXIT

# Prepare the build environment image.
docker build \
  --platform "$PLATFORM" \
  --build-arg BASE="$BUILDER_CONTAINER_BASE" \
  --build-arg PROTOC_VERSION="$PROTOC_VERSION" \
  --file docker/Dockerfile.builder \
  --tag "$BUILDER_CONTAINER_TAG" \
  "$TMP_BUILDER_DATA_DIR"

# Prepare the host paths so that they are owned by the proper user.
mkdir -p \
  "${BUILD_VOLUMES_PATH}/cargo" \
  "${BUILD_VOLUMES_PATH}/rustup" \
  "${HOST_TARGET}"

# Run the build in the docker environment.
docker run \
  --rm \
  -i \
  --platform "$PLATFORM" \
  --mount "type=bind,src=$(pwd),dst=/build" \
  --volume "${BUILD_VOLUMES_PATH}/cargo:/cargo-host:rw" \
  --volume "${BUILD_VOLUMES_PATH}/rustup:/rustup-host:rw" \
  --volume "${HOST_TARGET}:/build/target:rw" \
  --env CARGO_HOME=/cargo-host \
  --env RUSTUP_HOME=/rustup-host \
  --workdir /build \
  --user "$(id -u):$(id -g)" \
  "$BUILDER_CONTAINER_TAG" \
  cargo build --release

# Prepare artifacts.
TMP_ARTIFACTS_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_ARTIFACTS_DIR"' EXIT

# Define a list of all the binary targets to copy over.
BIN_TARGETS=(
  humanode-peer
  robonode-server
  robonode-keygen
)

# Populate the context for the runtime image build.
for TARGET in "${BIN_TARGETS[@]}"; do
  cp -t "$TMP_ARTIFACTS_DIR" "$HOST_TARGET/release/${TARGET}"
done

# Build docker image.
docker build \
  --platform "$PLATFORM" \
  --build-arg BASE="$RUNTIME_CONTAINER_BASE" \
  --file docker/Dockerfile.runtime \
  --tag "$RUNTIME_CONTAINER_TAG" \
  "$TMP_ARTIFACTS_DIR"
