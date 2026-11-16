# Copyright 2026 hrzlgnm
# SPDX-License-Identifier: MIT

FROM ubuntu:26.04@sha256:d66b366dac61b575bee2aef07e9c26e8070096fbe84a3d4d3eb0da069b5d76db
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    file \
    git \
    jq \
    libwebkit2gtk-4.1-dev \
    && rm -rf /var/lib/apt/lists/*

LABEL org.opencontainers.image.source="https://github.com/hrzlgnm/tauri-plugin-mdns"
LABEL org.opencontainers.image.title="tauri-plugin-mdns Ubuntu Builder"
