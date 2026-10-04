// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

//! Access to the native Android plugin layer.
//!
//! The only native surface the engine needs is the Android
//! `localNetwork` runtime permission (declared in the plugin manifest,
//! requested through the auto-implemented `checkPermissions` /
//! `requestPermissions` commands). iOS is not supported and there are
//! no plans to support it.

#[cfg(target_os = "ios")]
compile_error!("tauri-plugin-mdns does not support iOS; desktop and Android only.");

use super::{models::LocalNetworkState, LOCAL_NETWORK_ALIAS};
use std::collections::HashMap;

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

/// Handle to the native plugin layer.
pub struct Mdns<R: Runtime>(PluginHandle<R>);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<Mdns<R>, String> {
    let handle = api
        .register_android_plugin("com.hrzlgnm.mdns", "MdnsPlugin")
        .map_err(|e| {
            log::error!("failed to register Android mdns plugin: {e:?}");
            format!("failed to register Android mdns plugin: {e:?}")
        })?;
    Ok(Mdns(handle))
}

impl<R: Runtime> Mdns<R> {
    fn parse_state(state: &HashMap<String, String>) -> LocalNetworkState {
        state
            .get(LOCAL_NETWORK_ALIAS)
            .map(|state| LocalNetworkState::parse(state))
            .unwrap_or(LocalNetworkState::Denied)
    }

    /// Reports the native `localNetwork` permission state.
    pub fn local_network_state(&self) -> Result<LocalNetworkState, String> {
        self.0
            .run_mobile_plugin::<HashMap<String, String>>("checkPermissions", ())
            .map(|state| Self::parse_state(&state))
            .map_err(|e| {
                log::error!("failed to check local network permission: {e:?}");
                format!("failed to check local network permission: {e:?}")
            })
    }

    /// Requests the native `localNetwork` permission and reports the
    /// resulting state.
    pub fn request_local_network_access(&self) -> Result<LocalNetworkState, String> {
        self.0
            .run_mobile_plugin::<HashMap<String, String>>(
                "requestPermissions",
                serde_json::json!({ "permissions": [LOCAL_NETWORK_ALIAS] }),
            )
            .map(|state| Self::parse_state(&state))
            .map_err(|e| {
                log::error!("failed to request local network permission: {e:?}");
                format!("failed to request local network permission: {e:?}")
            })
    }

    /// Whether browsing may proceed given the current permission state.
    pub fn local_network_granted(&self) -> Result<bool, String> {
        Ok(self.local_network_state()?.is_granted())
    }

    /// Holds the Wi-Fi multicast lock so mDNS packets keep flowing while
    /// browsing. Best effort: logs but never fails the browse, as some
    /// devices still resolve without the lock in the foreground.
    pub fn acquire_multicast_lock(&self) {
        if let Err(e) = self.0.run_mobile_plugin::<()>("acquireMulticastLock", ()) {
            log::warn!("failed to acquire multicast lock: {e:?}, continuing anyway");
        }
    }

    /// Releases the Wi-Fi multicast lock once nothing browses anymore.
    /// Best effort: logs but never fails the stop.
    pub fn release_multicast_lock(&self) {
        if let Err(e) = self.0.run_mobile_plugin::<()>("releaseMulticastLock", ()) {
            log::warn!("failed to release multicast lock: {e:?}, continuing anyway");
        }
    }
}
