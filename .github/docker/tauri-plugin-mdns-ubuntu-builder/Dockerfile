# Copyright 2026 hrzlgnm
# SPDX-License-Identifier: MIT

FROM ubuntu:26.04@sha256:065aa8626841f16057b78b027b464cd59c4fac36c204c72dcbf48d66f5fa8a96
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
