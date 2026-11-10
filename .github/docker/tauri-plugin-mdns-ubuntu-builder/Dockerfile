# Copyright 2026 hrzlgnm
# SPDX-License-Identifier: MIT

FROM ubuntu:26.04@sha256:6a06111c3b0a8600feadb270aec771dc239bf407b25625f93f44ab00e11f4d3c
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
LABEL org.opencontainers.image.base.name="ubuntu:22.04"
