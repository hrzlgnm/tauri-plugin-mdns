// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

//! mDNS browsing engine: daemon lifecycle, browse tasks, interface
//! management, and event emission.
//!
//! Ported from the `mdns-browser` application backend so both apps share
//! one implementation.

use crate::models::*;
use mdns_sd::{Error, IfKind, ServiceDaemon, ServiceEvent};
#[cfg(not(windows))]
use pnet::datalink;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    net::IpAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager, Runtime, Window};

type SharedServiceDaemon = Arc<Mutex<ServiceDaemon>>;

const BROWSE_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(20);
const BROWSE_RETRY_ATTEMPTS: usize = 100;
const MDNS_SD_META_SERVICE: &str = "_services._dns-sd._udp.local.";
const MDNS_SD_IP_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
const METRICS_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
const INTERFACES_LIST_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
const VERIFY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

async fn browse_with_retry(
    daemon: &ServiceDaemon,
    service_type: &str,
) -> Result<mdns_sd::Receiver<ServiceEvent>, Error> {
    let mut attempts = 0;
    loop {
        match daemon.browse(service_type) {
            Ok(rx) => return Ok(rx),
            Err(Error::Again) if attempts < BROWSE_RETRY_ATTEMPTS => {
                attempts += 1;
                log::warn!(
                    "Failed to start browsing for {service_type}, retrying ({attempts}/{BROWSE_RETRY_ATTEMPTS})"
                );
                tokio::time::sleep(BROWSE_RETRY_DELAY).await;
            }
            Err(e) => {
                log::error!(
                    "Giving up browsing for {service_type} after {attempts} retries: {e:?}"
                );
                return Err(e);
            }
        }
    }
}

pub struct ManagedState {
    daemon: SharedServiceDaemon,
    /// Active instance-browse service types. Its mutex doubles as the
    /// browse-lifecycle coordination lock: every transition that starts
    /// or stops discovery holds it across the state change *and* the
    /// native multicast-lock call, so a stop's check-and-release cannot
    /// interleave with a concurrent start's insert-and-acquire.
    queriers: Arc<Mutex<HashSet<String>>>,
    meta_browsing: Arc<AtomicBool>,
    /// Generation counter for service-type browsing. Each `browse_types`
    /// call bumps it, so a superseded browse task ending later cannot
    /// clear fresh state or release the lock out from under it.
    meta_gen: Arc<AtomicU64>,
    metrics_subscribed: AtomicBool,
    interfaces_subscribed: AtomicBool,
    ipv4_enabled: AtomicBool,
    ipv6_enabled: AtomicBool,
    disabled_interfaces: Arc<Mutex<HashSet<String>>>,
}

impl ManagedState {
    pub fn new() -> Self {
        Self {
            daemon: initialize_shared_daemon(),
            queriers: Arc::new(Mutex::new(HashSet::new())),
            meta_browsing: Arc::new(AtomicBool::new(false)),
            meta_gen: Arc::new(AtomicU64::new(0)),
            metrics_subscribed: AtomicBool::new(false),
            interfaces_subscribed: AtomicBool::new(false),
            ipv4_enabled: AtomicBool::new(true),
            ipv6_enabled: AtomicBool::new(true),
            disabled_interfaces: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl Default for ManagedState {
    fn default() -> Self {
        Self::new()
    }
}

fn initialize_shared_daemon() -> SharedServiceDaemon {
    let daemon = ServiceDaemon::new().expect("Failed to create daemon");
    if let Err(err) = daemon.set_ip_check_interval(MDNS_SD_IP_CHECK_INTERVAL.as_secs() as u32) {
        log::warn!("Failed to set ip check interval: {err:?}, continuing anyway");
    }
    if let Err(err) = daemon.disable_interface(enumerate_mdns_incapable_interfaces()) {
        log::warn!("Failed to disable interface: {err:?}, continuing anyway");
    }
    Arc::new(Mutex::new(daemon))
}

fn convert_interface_id(id: &mdns_sd::InterfaceId) -> InterfaceScope {
    InterfaceScope {
        name: id.name.clone(),
        index: id.index,
    }
}

fn convert_to_scoped_addr(host_ip: &mdns_sd::ScopedIp) -> ScopedAddr {
    match host_ip {
        mdns_sd::ScopedIp::V4(host_ip_v4) => {
            let interfaces: BTreeSet<InterfaceScope> = host_ip_v4
                .interface_ids()
                .iter()
                .map(convert_interface_id)
                .collect();
            ScopedAddr {
                addr: host_ip.to_ip_addr(),
                interfaces,
                scope_id: None,
            }
        }
        mdns_sd::ScopedIp::V6(host_ip_v6) => {
            let interface = convert_interface_id(host_ip_v6.scope_id());
            let ip_addr = host_ip.to_ip_addr();
            let is_link_local = matches!(ip_addr, IpAddr::V6(v6) if v6.is_unicast_link_local());
            let scope_id = if is_link_local {
                #[cfg(windows)]
                {
                    Some(interface.index.to_string())
                }
                #[cfg(not(windows))]
                {
                    Some(interface.name.clone())
                }
            } else {
                None
            };
            let interfaces = BTreeSet::from([interface]);
            ScopedAddr {
                addr: ip_addr,
                interfaces,
                scope_id,
            }
        }
        _ => unreachable!(),
    }
}

fn from_resolved_service(resolved: &mdns_sd::ResolvedService) -> ResolvedService {
    let addresses: Vec<ScopedAddr> = resolved
        .addresses
        .iter()
        .map(convert_to_scoped_addr)
        .collect();

    let mut consolidated: Vec<ScopedAddr> = Vec::new();
    for addr in addresses {
        let is_ipv6_link_local = matches!(addr.addr, IpAddr::V6(v6) if v6.is_unicast_link_local());
        if is_ipv6_link_local {
            consolidated.push(addr);
        } else if let Some(existing) = consolidated.iter_mut().find(|a| a.addr == addr.addr) {
            existing.interfaces.extend(addr.interfaces);
        } else {
            consolidated.push(addr);
        }
    }
    consolidated.sort();

    let mut sorted_txt: Vec<TxtRecord> = resolved
        .txt_properties
        .iter()
        .map(|r| TxtRecord {
            key: r.key().into(),
            val: bytes_option_to_string_option_with_escaping(r.val()),
        })
        .collect();
    sorted_txt.sort_by(|a, b| a.key.cmp(&b.key));
    ResolvedService {
        instance_fullname: resolved.fullname.clone(),
        service_type: resolved.ty_domain.clone(),
        hostname: resolved.host.clone(),
        port: resolved.port,
        addresses: consolidated,
        subtype: resolved.sub_ty_domain.clone(),
        txt: sorted_txt,
        updated_at_micros: timestamp_micros(),
        dead: false,
    }
}

/// Emits an event to the window, logging (but not propagating) failures.
fn emit_event<R: Runtime, T>(window: &Window<R>, event: &str, payload: &T)
where
    T: serde::Serialize + std::fmt::Debug,
{
    log::trace!("Emitting event: {event} with payload: {payload:#?}");
    if let Err(e) = window.emit(event, payload) {
        log::error!("Failed to emit {event} event: {e:?}");
    }
}

/// Acquires the native Wi-Fi multicast lock (no-op off Android).
/// Call only while holding the coordination lock.
fn multicast_acquire<R: Runtime>(window: &Window<R>) {
    window
        .state::<crate::Mdns<R>>()
        .inner()
        .acquire_multicast_lock();
}

/// Releases the native Wi-Fi multicast lock (no-op off Android).
/// Call only while holding the coordination lock.
fn multicast_release<R: Runtime>(window: &Window<R>) {
    window
        .state::<crate::Mdns<R>>()
        .inner()
        .release_multicast_lock();
}

/// Reconciles service-type browsing state when its task ends (startup
/// failure or channel end), then releases the native lock when nothing
/// browses anymore. A task superseded by a newer `browse_types` call
/// leaves state alone: the newer browse owns the flag and the lock.
fn reconcile_meta_exit<R: Runtime>(
    window: &Window<R>,
    queriers: &Arc<Mutex<HashSet<String>>>,
    meta_browsing: &Arc<AtomicBool>,
    meta_gen: &Arc<AtomicU64>,
    generation: u64,
) {
    match queriers.lock() {
        Ok(queriers) => {
            if meta_gen.load(Ordering::SeqCst) != generation {
                return;
            }
            meta_browsing.store(false, Ordering::SeqCst);
            if queriers.is_empty() {
                multicast_release(window);
            }
        }
        Err(err) => {
            log::error!("Failed to lock running queriers: {err:?}");
        }
    }
}

/// Drops a failed instance browse and releases the native lock when
/// nothing browses anymore.
fn reconcile_querier_exit<R: Runtime>(
    window: &Window<R>,
    queriers: &Arc<Mutex<HashSet<String>>>,
    meta_browsing: &Arc<AtomicBool>,
    service_type: &str,
) {
    match queriers.lock() {
        Ok(mut queriers) => {
            queriers.remove(service_type);
            if queriers.is_empty() && !meta_browsing.load(Ordering::SeqCst) {
                multicast_release(window);
            }
        }
        Err(err) => {
            log::error!("Failed to lock running queriers: {err:?}");
        }
    }
}

/// Starts service-type discovery on the meta service, emitting
/// `service-type-found` for each valid type.
pub fn browse_types<R: Runtime>(window: Window<R>, state: &ManagedState) -> Result<(), String> {
    let daemon = state
        .daemon
        .lock()
        .map_err(|e| format!("Failed to lock daemon: {e:?}"))?;

    daemon
        .stop_browse(MDNS_SD_META_SERVICE)
        .map_err(|e| format!("Failed to stop browsing for {MDNS_SD_META_SERVICE}: {e:?}"))?;
    state.meta_browsing.store(true, Ordering::SeqCst);
    let generation = state.meta_gen.fetch_add(1, Ordering::SeqCst) + 1;

    let daemon = daemon.clone();
    let queriers = state.queriers.clone();
    let meta_browsing = state.meta_browsing.clone();
    let meta_gen = state.meta_gen.clone();
    {
        let _guard = state
            .queriers
            .lock()
            .map_err(|e| format!("Failed to lock running queriers: {e:?}"))?;
        multicast_acquire(&window);
    }
    tauri::async_runtime::spawn(async move {
        let receiver = match browse_with_retry(&daemon, MDNS_SD_META_SERVICE).await {
            Ok(receiver) => receiver,
            Err(_) => {
                reconcile_meta_exit(&window, &queriers, &meta_browsing, &meta_gen, generation);
                return;
            }
        };
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceFound(_service_type, full_name) => {
                    match check_service_type_fully_qualified(full_name.as_str()) {
                        Ok(MdnsLabelType::ServiceType) => {
                            emit_event(
                                &window,
                                "service-type-found",
                                &ServiceTypeFoundEvent {
                                    service_type: full_name,
                                },
                            );
                        }
                        Ok(MdnsLabelType::SubType) => {
                            log::debug!(
                                "Ignoring subtype `{full_name}` found during service type browsing"
                            );
                        }
                        Err(e) => {
                            log::debug!("Ignoring invalid service type `{full_name}`: {e}")
                        }
                    }
                }
                ServiceEvent::SearchStopped(service_type)
                    if service_type == MDNS_SD_META_SERVICE =>
                {
                    break;
                }
                _ => {}
            }
        }
        reconcile_meta_exit(&window, &queriers, &meta_browsing, &meta_gen, generation);
    });
    Ok(())
}

/// Stops all running instance browses.
///
/// Service-type discovery keeps running: it is the persistent watch that
/// tells frontends which types to browse, and the Wi-Fi multicast lock
/// stays held while it (or any instance browse) is active.
///
/// Entries whose daemon stop fails are kept and the failure is
/// propagated, so the lock is never released while browsing may still
/// be active. (`mdns-sd` only fails the stop itself on a full command
/// channel or a dead daemon, never for an unknown service type.)
pub fn stop_browse<R: Runtime>(window: &Window<R>, state: &ManagedState) -> Result<(), String> {
    let daemon = state
        .daemon
        .lock()
        .map_err(|e| format!("Failed to lock daemon: {e:?}"))?;
    let mut queriers = state
        .queriers
        .lock()
        .map_err(|e| format!("Failed to lock running queriers: {e:?}"))?;
    let mut failed = Vec::new();
    for ty_domain in queriers.iter() {
        if let Err(e) = daemon.stop_browse(ty_domain) {
            log::error!("Failed to stop browsing for {ty_domain}: {e:?}");
            failed.push(ty_domain.clone());
        }
    }
    if !failed.is_empty() {
        queriers.retain(|ty_domain| failed.contains(ty_domain));
        return Err(format!("Failed to stop browsing for {}", failed.join(", ")));
    }

    queriers.clear();
    if !state.meta_browsing.load(Ordering::SeqCst) {
        multicast_release(window);
    }
    Ok(())
}

/// Starts instance browsing for each given service type, emitting
/// `service-resolved` and `service-removed`.
pub fn browse_many<R: Runtime>(
    service_types: Vec<String>,
    window: Window<R>,
    state: &ManagedState,
) {
    let daemon = match state.daemon.lock() {
        Ok(daemon) => daemon.clone(),
        Err(err) => {
            log::error!("Failed to lock daemon: {err:?}");
            return;
        }
    };
    let fresh: Vec<String> = match state.queriers.lock() {
        Ok(mut queriers) => {
            let mut fresh = Vec::new();
            for service_type in service_types {
                if queriers.insert(service_type.clone()) {
                    fresh.push(service_type);
                }
            }
            if !fresh.is_empty() {
                multicast_acquire(&window);
            }
            fresh
        }
        Err(err) => {
            log::error!("Failed to lock running queriers: {err:?}");
            return;
        }
    };

    for service_type in fresh {
        let daemon = daemon.clone();
        let queriers = state.queriers.clone();
        let meta_browsing = state.meta_browsing.clone();
        let window = window.clone();
        tauri::async_runtime::spawn(async move {
            let receiver = match browse_with_retry(&daemon, &service_type).await {
                Ok(receiver) => receiver,
                Err(_) => {
                    reconcile_querier_exit(&window, &queriers, &meta_browsing, &service_type);
                    return;
                }
            };

            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(resolved) => emit_event(
                        &window,
                        "service-resolved",
                        &ServiceResolvedEvent {
                            service: from_resolved_service(&resolved),
                        },
                    ),

                    ServiceEvent::ServiceRemoved(_service_type, instance_name) => {
                        emit_event(
                            &window,
                            "service-removed",
                            &ServiceRemovedEvent {
                                instance_name,
                                at_micros: timestamp_micros(),
                            },
                        );
                    }
                    ServiceEvent::SearchStopped(_service_type) => {
                        break;
                    }
                    _ => {}
                }
            }
        });
    }
}

/// Verifies that an instance is still present on the network.
pub fn verify(instance_fullname: String, state: &ManagedState) -> Result<(), String> {
    let daemon = state
        .daemon
        .lock()
        .map_err(|e| format!("Failed to lock daemon: {e:?}"))?;
    daemon
        .verify(instance_fullname.clone(), VERIFY_TIMEOUT)
        .map_err(|e| format!("Failed to verify {instance_fullname}: {e:?}"))?;
    Ok(())
}

#[cfg(not(windows))]
/// Checks whether a network interface is capable of mDNS discovery.
///
/// mDNS capable interfaces must have at least one IP address, must not be loopback or point to
/// point, must be running and support multicast and broadcast. On Android there are some `rmnet`
/// (remote network virtual) interfaces for cellular modems without a broadcast capability like
/// ethernet or wifi interfaces, and sometimes a `dummy0` interface without multicast capability,
/// both of which are therefore considered incapable.
fn is_mdns_capable_pnet(interface: &pnet::datalink::NetworkInterface) -> bool {
    !interface.ips.is_empty()
        && !interface.is_loopback()
        && !interface.is_point_to_point()
        && interface.is_multicast()
        && interface.is_broadcast()
        && interface.is_running()
}

#[cfg(not(windows))]
fn enumerate_mdns_incapable_interfaces() -> Vec<IfKind> {
    let interfaces = datalink::interfaces();
    interfaces
        .iter()
        .filter(|interface| {
            // Skip loopback and point to point outright as those are disabled by default and
            // never part of the mDNS interface selection.
            !interface.is_loopback()
                && !interface.is_point_to_point()
                && !is_mdns_capable_pnet(interface)
        })
        .map(|interface| IfKind::from(interface.name.as_str()))
        .collect()
}

#[cfg(not(windows))]
pub fn enumerate_mdns_capable_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces: Vec<NetworkInterface> = datalink::interfaces()
        .iter()
        .filter(|interface| {
            // The loopback interface is offered as a selection even though it is not multicast
            // capable, as it allows browsing mDNS services that are only advertised on loopback.
            is_mdns_capable_pnet(interface)
                || (interface.is_loopback() && !interface.ips.is_empty())
        })
        .map(|interface| NetworkInterface {
            name: interface.name.clone(),
            addresses: interface.ips.iter().map(|ip| ip.ip().to_string()).collect(),
            enabled: true,
        })
        .collect();
    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    interfaces
}

#[cfg(windows)]
fn enumerate_mdns_incapable_interfaces() -> Vec<IfKind> {
    if let Ok(adapters) = ipconfig::get_adapters() {
        adapters
            .iter()
            .filter_map(|adapter| {
                // Skip SoftwareLoopback, Tunnel, and Ppp interfaces as these
                // interface types are disabled by default.
                if matches!(
                    adapter.if_type(),
                    ipconfig::IfType::SoftwareLoopback
                        | ipconfig::IfType::Tunnel
                        | ipconfig::IfType::Ppp
                ) {
                    return None;
                }
                if adapter.ip_addresses().is_empty()
                    || adapter.oper_status() != ipconfig::OperStatus::IfOperStatusUp
                    || !(adapter.if_type() == ipconfig::IfType::EthernetCsmacd
                        || adapter.if_type() == ipconfig::IfType::Ieee80211)
                {
                    Some(IfKind::from(adapter.friendly_name()))
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    }
}

#[cfg(windows)]
pub fn enumerate_mdns_capable_interfaces() -> Vec<NetworkInterface> {
    let adapters = match ipconfig::get_adapters() {
        Ok(adapters) => adapters,
        Err(err) => {
            log::error!("Failed to get network adapters: {err:?}");
            return vec![];
        }
    };
    let mut interfaces: Vec<NetworkInterface> = adapters
        .iter()
        .filter(|adapter| {
            // The loopback adapter is offered as a selection even though it is not
            // multicast capable, as it allows browsing mDNS services that are only
            // advertised on loopback.
            (!adapter.ip_addresses().is_empty()
                && adapter.oper_status() == ipconfig::OperStatus::IfOperStatusUp
                && (adapter.if_type() == ipconfig::IfType::EthernetCsmacd
                    || adapter.if_type() == ipconfig::IfType::Ieee80211))
                || (adapter.if_type() == ipconfig::IfType::SoftwareLoopback
                    && !adapter.ip_addresses().is_empty())
        })
        .map(|adapter| NetworkInterface {
            name: adapter.friendly_name().to_string(),
            addresses: adapter
                .ip_addresses()
                .iter()
                .map(|ip| ip.to_string())
                .collect(),
            enabled: true,
        })
        .collect();
    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    interfaces
}

async fn poll_interfaces<R: Runtime>(
    window: Window<R>,
    disabled_interfaces: Arc<Mutex<HashSet<String>>>,
) {
    let mut current: Vec<NetworkInterface> = Vec::new();
    loop {
        let disabled = match disabled_interfaces.lock() {
            Ok(disabled) => disabled.clone(),
            Err(err) => {
                log::error!("Failed to lock disabled interfaces: {err:?}");
                return;
            }
        };
        let interfaces =
            set_interface_enabled_flags(enumerate_mdns_capable_interfaces(), &disabled);
        if interfaces != current {
            current = interfaces.clone();
            emit_event(
                &window,
                "interfaces-changed",
                &InterfacesChangedEvent { interfaces },
            );
        }
        tokio::time::sleep(INTERFACES_LIST_CHECK_INTERVAL).await;
    }
}

/// Starts (or refreshes) the periodic `interfaces-changed` emission.
pub fn subscribe_interfaces<R: Runtime>(
    window: Window<R>,
    state: &ManagedState,
) -> Result<(), String> {
    if state
        .interfaces_subscribed
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        tauri::async_runtime::spawn(poll_interfaces(window, state.disabled_interfaces.clone()));
        Ok(())
    } else {
        let disabled = match state.disabled_interfaces.lock() {
            Ok(disabled) => disabled.clone(),
            Err(err) => {
                log::error!("Failed to lock disabled interfaces: {err:?}");
                return Err(format!("Failed to lock disabled interfaces: {err:?}"));
            }
        };
        let interfaces =
            set_interface_enabled_flags(enumerate_mdns_capable_interfaces(), &disabled);
        emit_event(
            &window,
            "interfaces-changed",
            &InterfacesChangedEvent { interfaces },
        );
        Ok(())
    }
}

/// Starts the periodic `metrics-changed` emission (first subscription wins).
pub fn subscribe_metrics<R: Runtime>(window: Window<R>, state: &ManagedState) {
    // Avoid multiple subscriptions when the frontend is reloaded.
    if state
        .metrics_subscribed
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        if let Ok(daemon) = state.daemon.lock() {
            let daemon = daemon.clone();
            let mut old_metrics = HashMap::new();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Ok(metrics_receiver) = daemon.get_metrics() {
                        if let Ok(metrics) = metrics_receiver.recv_async().await {
                            if old_metrics == metrics {
                                continue;
                            }
                            emit_event(
                                &window,
                                "metrics-changed",
                                &MetricsChangedEvent {
                                    metrics: metrics.clone(),
                                },
                            );
                            old_metrics = metrics;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }

                    tokio::time::sleep(METRICS_CHECK_INTERVAL).await;
                }
            });
        }
    }
}

pub fn get_protocol_flags(state: &ManagedState) -> ProtocolFlags {
    let ipv4_enabled = state.ipv4_enabled.load(Ordering::SeqCst);
    let ipv6_enabled = state.ipv6_enabled.load(Ordering::SeqCst);
    ProtocolFlags {
        ipv4: ipv4_enabled,
        ipv6: ipv6_enabled,
    }
}

/// Applies the configured interface selections to the mDNS daemon.
///
/// The daemon applies selections in order with later selections taking precedence, so we first
/// disable all interfaces, then re-enable the IPv4/IPv6 families based on the protocol flags,
/// then disable the interfaces the user has unselected, and finally re-disable the interfaces
/// that are not capable of mDNS.
fn apply_interface_selections(
    daemon: &std::sync::MutexGuard<ServiceDaemon>,
    disabled_interfaces: &HashSet<String>,
    ipv4_enabled: bool,
    ipv6_enabled: bool,
) -> Result<(), String> {
    daemon
        .disable_interface(IfKind::All)
        .map_err(|e| format!("Failed to disable all interfaces: {e:?}"))?;
    if ipv4_enabled {
        daemon
            .enable_interface(IfKind::IPv4)
            .map_err(|e| format!("Failed to enable IPv4 interfaces: {e:?}"))?;
    }
    if ipv6_enabled {
        daemon
            .enable_interface(IfKind::IPv6)
            .map_err(|e| format!("Failed to enable IPv6 interfaces: {e:?}"))?;
    }
    for name in disabled_interfaces {
        daemon
            .disable_interface(IfKind::Name(name.clone()))
            .map_err(|e| format!("Failed to disable {name} interface: {e:?}"))?;
    }
    // We have to disable interfaces that are not capable of mDNS, as enabling IPv4 or IPv6 may
    // also enable those interfaces.
    if let Err(err) = daemon.disable_interface(enumerate_mdns_incapable_interfaces()) {
        // Log the error but continue, as this is not critical.
        log::warn!("Failed to disable interfaces: {err:?}, continuing anyway");
    }
    Ok(())
}

/// Marks the given interfaces as enabled based on the provided set of disabled interface names.
fn set_interface_enabled_flags(
    mut interfaces: Vec<NetworkInterface>,
    disabled: &HashSet<String>,
) -> Vec<NetworkInterface> {
    for interface in &mut interfaces {
        interface.enabled = !disabled.contains(&interface.name);
    }
    interfaces
}

pub fn set_protocol_flags(state: &ManagedState, flags: ProtocolFlags) -> Result<(), String> {
    let disabled_interfaces = state
        .disabled_interfaces
        .lock()
        .map_err(|e| format!("Failed to lock disabled interfaces: {e:?}"))?
        .clone();
    let daemon = state
        .daemon
        .lock()
        .map_err(|e| format!("Failed to lock daemon: {e:?}"))?;
    if let Err(err) =
        apply_interface_selections(&daemon, &disabled_interfaces, flags.ipv4, flags.ipv6)
    {
        log::error!("Failed to apply interface selections: {err}");
        return Err(err);
    }
    state.ipv4_enabled.store(flags.ipv4, Ordering::SeqCst);
    state.ipv6_enabled.store(flags.ipv6, Ordering::SeqCst);
    Ok(())
}

pub fn set_interfaces(state: &ManagedState, enabled: Vec<String>) -> Result<(), String> {
    let enabled: HashSet<String> = enabled.into_iter().collect();
    let capable_names: HashSet<String> = enumerate_mdns_capable_interfaces()
        .into_iter()
        .map(|interface| interface.name)
        .collect();
    let new_disabled = capable_names
        .difference(&enabled)
        .cloned()
        .collect::<HashSet<_>>();

    let mut disabled_interfaces = state
        .disabled_interfaces
        .lock()
        .map_err(|e| format!("Failed to lock disabled interfaces: {e:?}"))?;
    if *disabled_interfaces == new_disabled {
        return Ok(());
    }

    let daemon = state
        .daemon
        .lock()
        .map_err(|e| format!("Failed to lock daemon: {e:?}"))?;
    let ipv4_enabled = state.ipv4_enabled.load(Ordering::SeqCst);
    let ipv6_enabled = state.ipv6_enabled.load(Ordering::SeqCst);
    if let Err(err) = apply_interface_selections(&daemon, &new_disabled, ipv4_enabled, ipv6_enabled)
    {
        log::error!("Failed to apply interface selections: {err}");
        return Err(err);
    }
    *disabled_interfaces = new_disabled;
    Ok(())
}

#[cfg(test)]
mod interface_selection_tests {
    use super::{set_interface_enabled_flags, NetworkInterface};
    use std::collections::HashSet;

    fn sample_interfaces() -> Vec<NetworkInterface> {
        vec![
            NetworkInterface {
                name: "en0".to_string(),
                addresses: vec!["192.168.1.1".to_string()],
                enabled: true,
            },
            NetworkInterface {
                name: "en1".to_string(),
                addresses: vec![],
                enabled: true,
            },
        ]
    }

    #[test]
    fn test_set_interface_enabled_flags_keeps_all_enabled_when_nothing_disabled() {
        let result = set_interface_enabled_flags(sample_interfaces(), &HashSet::new());
        assert!(result.iter().all(|interface| interface.enabled));
    }

    #[test]
    fn test_set_interface_enabled_flags_disables_matching_interfaces() {
        let disabled = HashSet::from(["en0".to_string()]);
        let result = set_interface_enabled_flags(sample_interfaces(), &disabled);
        assert!(
            !result
                .iter()
                .find(|interface| interface.name == "en0")
                .expect("To contain en0")
                .enabled
        );
        assert!(
            result
                .iter()
                .find(|interface| interface.name == "en1")
                .expect("To contain en1")
                .enabled
        );
    }
}

#[cfg(all(test, not(windows)))]
mod capable_interface_tests {
    use super::{enumerate_mdns_capable_interfaces, enumerate_mdns_incapable_interfaces};
    use mdns_sd::IfKind;
    use pnet::datalink;

    #[test]
    fn test_loopback_not_included_in_mdns_incapable_interfaces() {
        let result = enumerate_mdns_incapable_interfaces();
        // Gather actual loopback interface names on this system.
        let loopback_names: std::collections::HashSet<String> = {
            datalink::interfaces()
                .into_iter()
                .filter(|iface| iface.is_loopback())
                .map(|iface| iface.name)
                .collect()
        };
        assert!(
            !loopback_names.is_empty(),
            "No loopback interfaces detected on this host; test cannot validate exclusion"
        );
        let any_loopback_found = result.iter().any(|ifkind| match ifkind {
            IfKind::Name(name) => loopback_names.contains(name),
            _ => false,
        });
        assert!(
            !any_loopback_found,
            "Loopback interfaces {:?} should not be included in mdns-incapable interfaces",
            loopback_names
        );
    }

    #[test]
    fn test_loopback_included_in_mdns_capable_interfaces() {
        let result = enumerate_mdns_capable_interfaces();
        // Gather actual loopback interface names on this system.
        let loopback_names: std::collections::HashSet<String> = {
            datalink::interfaces()
                .into_iter()
                .filter(|iface| iface.is_loopback() && !iface.ips.is_empty())
                .map(|iface| iface.name)
                .collect()
        };
        assert!(
            !loopback_names.is_empty(),
            "No loopback interfaces detected on this host; test cannot validate inclusion"
        );
        let any_loopback_found = result
            .iter()
            .any(|interface| loopback_names.contains(&interface.name));
        assert!(
            any_loopback_found,
            "Loopback interfaces {:?} should be included in mdns-capable interfaces",
            loopback_names
        );
    }
}
