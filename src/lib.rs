// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

//! # tauri-plugin-mdns
//!
//! mDNS service discovery (browse) for Tauri apps on desktop and Android,
//! backed by the [`mdns-sd`] crate in Rust on every supported platform.
//! iOS is not supported and there are no plans to support it.
//!
//! The plugin registers its commands under the `plugin:mdns|` namespace;
//! grant the plugin's `default` permission in the app's capabilities so
//! the frontend can invoke them. Discovery events (`service-type-found`,
//! `service-resolved`, `service-removed`, `interfaces-changed`,
//! `metrics-changed`) are emitted to the calling window.
//!
//! On Android the engine additionally handles the `ACCESS_LOCAL_NETWORK`
//! runtime permission, which Android 17 mandates for apps targeting SDK
//! 37+: without it local-network access — including mDNS — is blocked by
//! default, so [`commands::browse_types`] and [`commands::browse_many`]
//! refuse to start while access is not granted. Frontends drive the
//! consent flow through [`commands::local_network_status`] and
//! [`commands::request_local_network_access`]; the declaration travels in
//! the plugin manifest and is merged into the host app, surviving
//! `tauri android init`. While browsing, the plugin holds a Wi-Fi
//! multicast lock (via `CHANGE_WIFI_MULTICAST_STATE`) so multicast
//! packets keep flowing; it is released once nothing browses anymore.
//!
//! [`mdns-sd`]: https://docs.rs/mdns-sd

use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
#[cfg(desktop)]
mod desktop;
mod engine;
#[cfg(mobile)]
mod mobile;
mod models;

#[cfg(desktop)]
pub use desktop::Mdns;
#[cfg(mobile)]
pub use mobile::Mdns;

pub use models::*;

/// Alias of the Android runtime permission declared on the Kotlin plugin
/// class (`android.permission.ACCESS_LOCAL_NETWORK`).
pub const LOCAL_NETWORK_ALIAS: &str = "localNetwork";

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`]
/// to access the mDNS APIs.
pub trait MdnsExt<R: Runtime> {
    fn mdns(&self) -> &Mdns<R>;
}

impl<R: Runtime, T: Manager<R>> MdnsExt<R> for T {
    fn mdns(&self) -> &Mdns<R> {
        self.state::<Mdns<R>>().inner()
    }
}

/// Initializes the plugin.
///
/// Manages the browsing engine state and registers the commands under
/// their `plugin:mdns|` names; the app must grant the plugin's `default`
/// permission.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::<R>::new("mdns")
        .invoke_handler(tauri::generate_handler![
            commands::browse_many,
            commands::browse_types,
            commands::get_protocol_flags,
            commands::local_network_status,
            commands::request_local_network_access,
            commands::set_interfaces,
            commands::set_protocol_flags,
            commands::stop_browse,
            commands::subscribe_interfaces,
            commands::subscribe_metrics,
            commands::verify,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let mdns = mobile::init(app, api)?;
            #[cfg(desktop)]
            let mdns = desktop::init(app, api)?;
            app.manage(mdns);
            app.manage(engine::ManagedState::new());
            Ok(())
        })
        .build()
}
