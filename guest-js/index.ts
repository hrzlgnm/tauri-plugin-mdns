// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** A discovered TXT record: `key` alone, or `key=value`. */
export interface TxtRecord {
    key: string;
    val: string | null;
}

/** A network interface a resolved address was seen on. */
export interface InterfaceScope {
    name: string;
    index: number;
}

/** A resolved address with the interfaces it was seen on. */
export interface ScopedAddr {
    addr: string;
    interfaces: Array<InterfaceScope>;
    scope_id?: string | null;
}

/** A fully resolved service instance. */
export interface ResolvedService {
    instance_fullname: string;
    service_type: string;
    hostname: string;
    port: number;
    addresses: Array<ScopedAddr>;
    subtype: string | null;
    txt: Array<TxtRecord>;
    /** Microseconds since the Unix epoch, as a string. */
    updated_at_micros: string;
    dead: boolean;
}

/** A network interface selectable for mDNS browsing. */
export interface NetworkInterface {
    name: string;
    addresses: Array<string>;
    enabled: boolean;
}

/** Enabled IP protocol families. */
export interface ProtocolFlags {
    ipv4: boolean;
    ipv6: boolean;
}

/** Payload of the `service-type-found` event. */
export interface ServiceTypeFoundEvent {
    service_type: string;
}

/** Payload of the `service-resolved` event. */
export interface ServiceResolvedEvent {
    service: ResolvedService;
}

/** Payload of the `service-removed` event. */
export interface ServiceRemovedEvent {
    instance_name: string;
    /** Microseconds since the Unix epoch, as a string. */
    at_micros: string;
}

/** Payload of the `interfaces-changed` event. */
export interface InterfacesChangedEvent {
    interfaces: Array<NetworkInterface>;
}

/** Payload of the `metrics-changed` event. */
export interface MetricsChangedEvent {
    metrics: Record<string, number>;
}

/**
 * Native `localNetwork` permission state, as reported by
 * {@link localNetworkStatus} and {@link requestLocalNetworkAccess}.
 * Always `granted` off Android; on Android 17+ (targeting SDK 37) one of
 * the four states.
 */
export type LocalNetworkState = 'granted' | 'denied' | 'prompt' | 'prompt-with-rationale';

/**
 * Starts service-type discovery, emitting `service-type-found` events.
 *
 * Refuses while local-network access is not granted (Android 17+
 * targeting SDK 37): check {@link localNetworkStatus} first and drive
 * the consent flow through {@link requestLocalNetworkAccess}.
 */
export async function browseTypes(): Promise<void> {
    return invoke('plugin:mdns|browse_types');
}

/**
 * Starts instance browsing for each given service type, emitting
 * `service-resolved` and `service-removed` events.
 *
 * Refuses while local-network access is not granted (Android 17+
 * targeting SDK 37): check {@link localNetworkStatus} first and drive
 * the consent flow through {@link requestLocalNetworkAccess}.
 */
export async function browseMany(serviceTypes: Array<string>): Promise<void> {
    return invoke('plugin:mdns|browse_many', { serviceTypes });
}

/** Stops all running instance browses. */
export async function stopBrowse(): Promise<void> {
    return invoke('plugin:mdns|stop_browse');
}

/** Verifies that an instance is still present on the network. */
export async function verifyInstance(instanceFullname: string): Promise<void> {
    return invoke('plugin:mdns|verify', { instanceFullname });
}

/** Starts (or refreshes) the periodic `interfaces-changed` emission. */
export async function subscribeInterfaces(): Promise<void> {
    return invoke('plugin:mdns|subscribe_interfaces');
}

/** Starts the periodic `metrics-changed` emission (first call wins). */
export async function subscribeMetrics(): Promise<void> {
    return invoke('plugin:mdns|subscribe_metrics');
}

/** Reports the enabled IP protocol families. */
export async function getProtocolFlags(): Promise<ProtocolFlags> {
    return invoke('plugin:mdns|get_protocol_flags');
}

/** Applies the enabled IP protocol families. */
export async function setProtocolFlags(flags: ProtocolFlags): Promise<void> {
    return invoke('plugin:mdns|set_protocol_flags', { flags });
}

/** Applies the enabled network interfaces by name. */
export async function setInterfaces(enabled: Array<string>): Promise<void> {
    return invoke('plugin:mdns|set_interfaces', { enabled });
}

/**
 * Reports the native `localNetwork` permission state.
 * Call before browsing; gate discovery on a blocking denied state.
 */
export async function localNetworkStatus(): Promise<LocalNetworkState> {
    return invoke('plugin:mdns|local_network_status');
}

/**
 * Requests the native `localNetwork` permission and reports the
 * resulting state. Call from a user gesture after showing a
 * rationale for the non-granted states of {@link localNetworkStatus}.
 */
export async function requestLocalNetworkAccess(): Promise<LocalNetworkState> {
    return invoke('plugin:mdns|request_local_network_access');
}

/** Listens for discovered service types (after {@link browseTypes}). */
export function onServiceTypeFound(
    cb: (payload: ServiceTypeFoundEvent) => void
): Promise<UnlistenFn> {
    return listen<ServiceTypeFoundEvent>('service-type-found', (event) => cb(event.payload));
}

/** Listens for resolved service instances (after {@link browseMany}). */
export function onServiceResolved(
    cb: (payload: ServiceResolvedEvent) => void
): Promise<UnlistenFn> {
    return listen<ServiceResolvedEvent>('service-resolved', (event) => cb(event.payload));
}

/** Listens for removed service instances (after {@link browseMany}). */
export function onServiceRemoved(
    cb: (payload: ServiceRemovedEvent) => void
): Promise<UnlistenFn> {
    return listen<ServiceRemovedEvent>('service-removed', (event) => cb(event.payload));
}

/** Listens for interface changes (after {@link subscribeInterfaces}). */
export function onInterfacesChanged(
    cb: (payload: InterfacesChangedEvent) => void
): Promise<UnlistenFn> {
    return listen<InterfacesChangedEvent>('interfaces-changed', (event) => cb(event.payload));
}

/** Listens for daemon metric changes (after {@link subscribeMetrics}). */
export function onMetricsChanged(
    cb: (payload: MetricsChangedEvent) => void
): Promise<UnlistenFn> {
    return listen<MetricsChangedEvent>('metrics-changed', (event) => cb(event.payload));
}
