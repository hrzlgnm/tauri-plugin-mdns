// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

//! Access to the native mobile plugin layer.
//!
//! The only native surface the engine needs is the Android
//! `localNetwork` runtime permission (declared in the plugin manifest,
//! requested through the auto-implemented `checkPermissions` /
//! `requestPermissions` commands). iOS registers no native plugin and
//! reports granted: local-network access there is covered by the app's
//! `Info.plist` keys.

use super::{models::LocalNetworkState, LOCAL_NETWORK_ALIAS};
use std::collections::HashMap;

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

/// Handle to the native plugin layer.
pub struct Mdns<R: Runtime>(Option<PluginHandle<R>>);

#[cfg(target_os = "android")]
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
    Ok(Mdns(Some(handle)))
}

#[cfg(target_os = "ios")]
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<Mdns<R>, String> {
    Ok(Mdns(None))
}

impl<R: Runtime> Mdns<R> {
    fn parse_state(state: &HashMap<String, String>) -> LocalNetworkState {
        state
            .get(LOCAL_NETWORK_ALIAS)
            .map(LocalNetworkState::parse)
            .unwrap_or(LocalNetworkState::Denied)
    }

    /// Reports the native `localNetwork` permission state.
    pub fn local_network_state(&self) -> Result<LocalNetworkState, String> {
        match &self.0 {
            Some(handle) => handle
                .run_mobile_plugin::<HashMap<String, String>>("checkPermissions", ())
                .map(|state| Self::parse_state(&state))
                .map_err(|e| {
                    log::error!("failed to check local network permission: {e:?}");
                    format!("failed to check local network permission: {e:?}")
                }),
            // No native layer (e.g. iOS): access is covered elsewhere.
            None => Ok(LocalNetworkState::Granted),
        }
    }

    /// Requests the native `localNetwork` permission and reports the
    /// resulting state.
    pub fn request_local_network_access(&self) -> Result<LocalNetworkState, String> {
        match &self.0 {
            Some(handle) => handle
                .run_mobile_plugin::<HashMap<String, String>>(
                    "requestPermissions",
                    serde_json::json!({ "permissions": [LOCAL_NETWORK_ALIAS] }),
                )
                .map(|state| Self::parse_state(&state))
                .map_err(|e| {
                    log::error!("failed to request local network permission: {e:?}");
                    format!("failed to request local network permission: {e:?}")
                }),
            // No native layer (e.g. iOS): nothing to request.
            None => Ok(LocalNetworkState::Granted),
        }
    }

    /// Whether browsing may proceed given the current permission state.
    pub fn local_network_granted(&self) -> Result<bool, String> {
        Ok(self.local_network_state()?.is_granted())
    }
}
