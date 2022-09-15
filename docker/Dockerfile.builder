# syntax=docker/dockerfile:1.2

ARG BASE

FROM ${BASE}

RUN apt-get update \
  && apt-get install -y \
  clang \
  protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*
