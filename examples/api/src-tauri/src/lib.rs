// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_mdns::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
