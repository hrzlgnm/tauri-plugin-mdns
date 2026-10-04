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
    // The android path wires the plugin's native module (manifest with the
    // ACCESS_LOCAL_NETWORK declaration, Kotlin permission layer) into the
    // host app: the build script emits DEP_*_ANDROID_LIBRARY_PATH, which
    // the app's tauri-build turns into tauri.settings.gradle. Without it
    // the Rust engine builds but native calls and the manifest merge
    // silently never happen.
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .build();
}
