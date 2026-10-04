// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &[
    "browse_many",
    "browse_types",
    "get_protocol_flags",
    "local_network_status",
    "request_local_network_access",
    "set_interfaces",
    "set_protocol_flags",
    "stop_browse",
    "subscribe_interfaces",
    "subscribe_metrics",
    "verify",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
