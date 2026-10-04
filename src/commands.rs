// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

//! Tauri commands exposed under the `plugin:mdns|` namespace.
//!
//! Browsing commands refuse to start while local-network access is not
//! granted (Android 17+ targeting SDK 37); frontends drive the consent
//! flow through [`local_network_status`] and
//! [`request_local_network_access`] and gate their UI on a blocking
//! denied state.

use tauri::{AppHandle, Runtime, State, Window};

use crate::{
    engine::{self, ManagedState},
    models::{LocalNetworkState, ProtocolFlags},
    Mdns,
};

/// Rejects browsing while local-network access is unavailable.
fn ensure_local_network_access<R: Runtime>(mdns: &Mdns<R>) -> Result<(), String> {
    if mdns.local_network_granted()? {
        Ok(())
    } else {
        log::error!("local network access denied, refusing to browse");
        Err(
            "local network access denied: grant nearby-devices access in system settings to browse"
                .to_string(),
        )
    }
}

/// Starts service-type discovery, emitting `service-type-found` events.
///
/// The engine holds the Android Wi-Fi multicast lock while browsing so
/// discovery stays alive; the lock is released once nothing browses
/// anymore.
#[tauri::command]
pub async fn browse_types<R: Runtime>(
    window: Window<R>,
    mdns: State<'_, Mdns<R>>,
    state: State<'_, ManagedState>,
) -> Result<(), String> {
    ensure_local_network_access(&mdns)?;
    engine::browse_types(window, &state)
}

/// Stops all running instance browses.
///
/// The engine releases the Android Wi-Fi multicast lock once nothing
/// browses anymore; service-type discovery keeps running (and the lock
/// stays held while it does).
#[tauri::command]
pub fn stop_browse<R: Runtime>(
    window: Window<R>,
    state: State<'_, ManagedState>,
) -> Result<(), String> {
    engine::stop_browse(&window, &state)
}

/// Verifies that an instance is still present on the network.
#[tauri::command]
pub fn verify(instance_fullname: String, state: State<'_, ManagedState>) -> Result<(), String> {
    engine::verify(instance_fullname, &state)
}

/// Starts instance browsing for each given service type, emitting
/// `service-resolved` and `service-removed` events.
///
/// The engine holds the Android Wi-Fi multicast lock while browsing so
/// discovery stays alive; the lock is released once nothing browses
/// anymore.
#[tauri::command]
pub async fn browse_many<R: Runtime>(
    service_types: Vec<String>,
    window: Window<R>,
    mdns: State<'_, Mdns<R>>,
    state: State<'_, ManagedState>,
) -> Result<(), String> {
    ensure_local_network_access(&mdns)?;
    engine::browse_many(service_types, window, &state);
    Ok(())
}

/// Starts (or refreshes) the periodic `interfaces-changed` emission.
#[tauri::command]
pub fn subscribe_interfaces<R: Runtime>(
    window: Window<R>,
    state: State<'_, ManagedState>,
) -> Result<(), String> {
    engine::subscribe_interfaces(window, &state)
}

/// Starts the periodic `metrics-changed` emission (first call wins).
#[tauri::command]
pub fn subscribe_metrics<R: Runtime>(window: Window<R>, state: State<'_, ManagedState>) {
    engine::subscribe_metrics(window, &state);
}

/// Reports the enabled IP protocol families.
#[tauri::command]
pub fn get_protocol_flags(state: State<'_, ManagedState>) -> ProtocolFlags {
    engine::get_protocol_flags(&state)
}

/// Applies the enabled IP protocol families.
#[tauri::command]
pub fn set_protocol_flags(
    state: State<'_, ManagedState>,
    flags: ProtocolFlags,
) -> Result<(), String> {
    engine::set_protocol_flags(&state, flags)
}

/// Applies the enabled network interfaces by name.
#[tauri::command]
pub fn set_interfaces(state: State<'_, ManagedState>, enabled: Vec<String>) -> Result<(), String> {
    engine::set_interfaces(&state, enabled)
}

/// Reports the native `localNetwork` permission state.
///
/// Always granted off Android; on Android 17+ (targeting SDK 37) one of
/// `granted`, `denied`, `prompt`, or `prompt-with-rationale`.
#[tauri::command]
pub fn local_network_status<R: Runtime>(
    _app: AppHandle<R>,
    mdns: State<'_, Mdns<R>>,
) -> Result<LocalNetworkState, String> {
    mdns.local_network_state()
}

/// Requests the native `localNetwork` permission and reports the
/// resulting state. Call from a user gesture after showing a
/// rationale for the non-granted states of [`local_network_status`].
#[tauri::command]
pub fn request_local_network_access<R: Runtime>(
    _app: AppHandle<R>,
    mdns: State<'_, Mdns<R>>,
) -> Result<LocalNetworkState, String> {
    mdns.request_local_network_access()
}
