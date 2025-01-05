# syntax=docker/dockerfile:1.2

ARG BASE

FROM ${BASE}

RUN apt-get update \
  && apt-get install -y \
  clang \
  unzip \
  && rm -rf /var/lib/apt/lists/*

ARG PROTOC_VERSION
RUN curl -Lo protoc.zip "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip" \
  && unzip -q protoc.zip bin/protoc -d /usr/local \
  && chmod a+x /usr/local/bin/protoc \
  && rm -rf protoc.zip
