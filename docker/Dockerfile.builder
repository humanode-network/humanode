# syntax=docker/dockerfile:1.2

ARG BASE

FROM ${BASE}

RUN apt-get update \
  && apt-get install -y \
  clang \
  unzip \
  && rm -rf /var/lib/apt/lists/*

RUN PROTOC_VERSION=$(curl -s "https://api.github.com/repos/protocolbuffers/protobuf/releases/latest" | grep -Po '"tag_name": "v\K[0-9.]+') \
  && curl -Lo protoc.zip "https://github.com/protocolbuffers/protobuf/releases/latest/download/protoc-${PROTOC_VERSION}-linux-x86_64.zip" \
  && unzip -q protoc.zip bin/protoc -d /usr/local \
  && chmod a+x /usr/local/bin/protoc \
  && rm -rf protoc.zip
