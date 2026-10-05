// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

//! Desktop access to the mDNS engine.
//!
//! Local-network access needs no runtime permission on desktop, so the
//! handle always reports granted.

use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use super::models::LocalNetworkState;

/// Handle to the mDNS engine on desktop.
///
/// The `fn() -> R` phantom (rather than `R` itself) keeps the handle
/// `Send + Sync` for any runtime, as `tauri::Manager::manage` requires.
pub struct Mdns<R: Runtime>(PhantomData<fn() -> R>);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<Mdns<R>, String> {
    Ok(Mdns(PhantomData))
}

impl<R: Runtime> Mdns<R> {
    /// Reports the native `localNetwork` permission state.
    pub fn local_network_state(&self) -> Result<LocalNetworkState, String> {
        Ok(LocalNetworkState::Granted)
    }

    /// Requests the native `localNetwork` permission and reports the
    /// resulting state.
    pub fn request_local_network_access(&self) -> Result<LocalNetworkState, String> {
        Ok(LocalNetworkState::Granted)
    }

    /// Whether browsing may proceed given the current permission state.
    pub fn local_network_granted(&self) -> Result<bool, String> {
        Ok(true)
    }
}
