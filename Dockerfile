# syntax=docker/dockerfile:1.23

ARG BUILDER_BASE=rust:bookworm
ARG RUNTIME_BASE=debian:bookworm

FROM ${BUILDER_BASE} AS builder

SHELL ["/bin/bash", "-c"]

RUN apt-get update \
  && apt-get install -y \
  clang \
  unzip \
  && rm -rf /var/lib/apt/lists/*

RUN --mount=source=depversions.sh,target=/depversions.sh \
  set -a && source /depversions.sh && set +a \
  && curl -Lo protoc.zip "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip" \
  && unzip -q protoc.zip bin/protoc -d /usr/local \
  && chmod a+x /usr/local/bin/protoc \
  && rm -rf protoc.zip

RUN mkdir -p ~/.ssh \
  && chmod 0600 ~/.ssh \
  && ssh-keyscan github.com >>~/.ssh/known_hosts

FROM ${RUNTIME_BASE} AS runtime

RUN apt-get update \
  && apt-get install -y \
  libssl3 \
  ca-certificates \
  jq \
  curl \
  && rm -rf /var/lib/apt/lists/*

FROM builder AS build

WORKDIR /worktree

# Install rust.
RUN \
  --mount=type=bind,target=rust-toolchain.toml,source=rust-toolchain.toml \
  --mount=type=cache,target=/usr/local/rustup \
  rustup install

# Build the binaries.
RUN \
  --mount=type=bind,target=.,readwrite \
  --mount=type=cache,target=/usr/local/rustup \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=target \
  --mount=type=ssh \
  RUST_BACKTRACE=1 \
  CARGO_TARGET_DIR=target/artifacts \
  cargo build --release --locked --workspace

# Copy artifacts.
RUN \
  --mount=type=cache,target=target \
  cp -r target/artifacts /artifacts \
  && ls -la /artifacts

FROM runtime AS runtime-release-artifact

ONBUILD ARG ARTIFACT
ONBUILD COPY --from=build "/artifacts/release/${ARTIFACT}" /usr/local/bin
ONBUILD RUN ldd "/usr/local/bin/${ARTIFACT}"

ARG ARTIFACT=robonode-server
FROM runtime-release-artifact AS robonode-server
CMD ["robonode-server"]

ARG ARTIFACT=robonode-keygen
FROM runtime-release-artifact AS robonode-keygen
CMD ["robonode-keygen"]

# Keep the peer last as the default target.
ARG ARTIFACT=humanode-peer
FROM runtime-release-artifact AS humanode-peer
CMD ["humanode-peer"]
