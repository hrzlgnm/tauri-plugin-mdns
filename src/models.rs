// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

//! Wire types for the mDNS discovery events and commands.
//!
//! The JSON shapes intentionally match the `mdns-browser` boundary types so
//! existing frontends port without payload changes: microsecond timestamps
//! travel as strings, structs use camelCase.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
    net::IpAddr,
    time::SystemTime,
};

pub type ServiceTypes = Vec<String>;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TxtRecord {
    pub key: String,
    pub val: Option<String>,
}

impl Display for TxtRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.val.is_none() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "{}={}", self.key, self.val.clone().expect("To exist"))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InterfaceScope {
    pub name: String,
    pub index: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedAddr {
    pub addr: IpAddr,
    pub interfaces: BTreeSet<InterfaceScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
}

impl From<IpAddr> for ScopedAddr {
    fn from(addr: IpAddr) -> Self {
        ScopedAddr {
            addr,
            interfaces: BTreeSet::new(),
            scope_id: None,
        }
    }
}

impl Display for ScopedAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.scope_id {
            write!(f, "{}%{}", self.addr, name)
        } else if self.interfaces.is_empty() {
            write!(f, "{}", self.addr)
        } else {
            let interface_names: Vec<&str> =
                self.interfaces.iter().map(|i| i.name.as_str()).collect();
            write!(f, "{} via {}", self.addr, interface_names.join(", "))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ResolvedService {
    pub instance_fullname: String,
    pub service_type: String,
    pub hostname: String,
    pub port: u16,
    pub addresses: Vec<ScopedAddr>,
    pub subtype: Option<String>,
    pub txt: Vec<TxtRecord>,
    #[serde(with = "serde_with::As::<serde_with::DisplayFromStr>")]
    pub updated_at_micros: u64,
    pub dead: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub name: String,
    pub addresses: Vec<String>,
    pub enabled: bool,
}

/// Event emitted when the set of mDNS-capable network interfaces changes.
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterfacesChangedEvent {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct MetricsChangedEvent {
    pub metrics: HashMap<String, i64>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServiceResolvedEvent {
    pub service: ResolvedService,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServiceTypeFoundEvent {
    pub service_type: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServiceRemovedEvent {
    pub instance_name: String,
    #[serde(with = "serde_with::As::<serde_with::DisplayFromStr>")]
    pub at_micros: u64,
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug)]
pub struct ProtocolFlags {
    pub ipv4: bool,
    pub ipv6: bool,
}

impl Default for ProtocolFlags {
    fn default() -> Self {
        Self {
            ipv4: true,
            ipv6: true,
        }
    }
}

/// States of the Android `localNetwork` permission alias, as reported by
/// the native plugin layer.
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum LocalNetworkState {
    Granted,
    Denied,
    Prompt,
    PromptWithRationale,
}

impl LocalNetworkState {
    /// Parses a state string from the native layer (`granted`, `denied`,
    /// `prompt`, `prompt-with-rationale`); unknown values deny.
    pub fn parse(state: &str) -> Self {
        match state {
            "granted" => Self::Granted,
            "denied" => Self::Denied,
            "prompt" => Self::Prompt,
            "prompt-with-rationale" => Self::PromptWithRationale,
            _ => Self::Denied,
        }
    }

    pub fn is_granted(&self) -> bool {
        matches!(self, Self::Granted)
    }
}

pub fn timestamp_micros() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    since_epoch.as_secs() * 1_000_000 + u64::from(since_epoch.subsec_micros())
}

fn string_with_control_characters_escaped(input: String) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_control() {
                format!(r"\u{:04x}", ch as u32)
            } else {
                ch.to_string()
            }
        })
        .collect()
}

pub fn bytes_option_to_string_option_with_escaping(maybe_bytes: Option<&[u8]>) -> Option<String> {
    maybe_bytes.map(|bytes| match String::from_utf8(bytes.to_vec()) {
        Ok(utf8_string) => string_with_control_characters_escaped(utf8_string),
        Err(_) => byte_array_hexlified(bytes),
    })
}

fn byte_array_hexlified(byte_array: &[u8]) -> String {
    byte_array
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join("")
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum MdnsError {
    #[error("The trailing dot is missing")]
    MissingTrailingDot,
    #[error("The service type label is not well formed")]
    InvalidService,
    #[error("The service sub type label is not well formed")]
    InvalidSubType,
    #[error("The service sub label is invalid, expected `_sub`")]
    InvalidSublabel,
    #[error("The protocol is invalid, expected `_tcp` or `_udp`")]
    InvalidProtocol,
    #[error("The domain is invalid, expected `.local.`")]
    InvalidDomain,
    #[error("The service type format is incorrect, expected to contain 3 or 5 parts")]
    IncorrectFormat,
}

#[derive(PartialEq, Eq, Debug)]
pub enum MdnsLabelType {
    ServiceType,
    SubType,
}

fn check_mdns_label(label: &str, is_subtype: bool) -> Result<MdnsLabelType, MdnsError> {
    let valid_dns_chars = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    let error = if is_subtype {
        MdnsError::InvalidSubType
    } else {
        MdnsError::InvalidService
    };

    if !label.starts_with('_') {
        return Err(error);
    }

    let label_content = &label[1..];

    // Ensure the label content doesn't start with an underscore
    if label_content.starts_with('_') {
        return Err(error);
    }

    // Ensure the label content doesn't end with an underscore
    if label_content.ends_with('_') {
        return Err(error);
    }

    if !label_content.chars().all(valid_dns_chars) {
        return Err(error);
    }

    // Ensure no double hyphens are present
    if label_content.contains("--") {
        return Err(error);
    }

    // Ensure the label does not start or end with a hyphen
    if label_content.starts_with('-') || label_content.ends_with('-') {
        return Err(error);
    }

    if is_subtype {
        Ok(MdnsLabelType::SubType)
    } else {
        Ok(MdnsLabelType::ServiceType)
    }
}

pub fn check_service_type_fully_qualified(service_type: &str) -> Result<MdnsLabelType, MdnsError> {
    // The service type must end with a trailing dot
    if !service_type.ends_with('.') {
        return Err(MdnsError::MissingTrailingDot);
    }

    // Remove the trailing dot for validation purposes
    let service_type = service_type.strip_suffix('.').expect("To end with .");

    // Split into parts based on dots
    let parts: Vec<&str> = service_type.split('.').collect();

    // Validate the number of parts for formats:
    // 1) _service._protocol.local
    // 2) _subtype._sub._service._protocol.local
    if parts.len() != 3 && parts.len() != 5 {
        return Err(MdnsError::IncorrectFormat);
    }

    let domain = parts.last().expect("To have a domain"); // Domain is always the last component
    let protocol = parts[parts.len() - 2]; // Protocol is the second-to-last component

    // Validate protocol name (must be either _tcp or _udp)
    if protocol != "_tcp" && protocol != "_udp" {
        return Err(MdnsError::InvalidProtocol);
    }

    // Validate domain (must be "local")
    if *domain != "local" {
        return Err(MdnsError::InvalidDomain);
    }

    // Validate service name
    let service = if parts.len() == 3 { parts[0] } else { parts[2] };
    check_mdns_label(service, false)?;

    // Validate optional subtype if present
    if parts.len() == 5 {
        let sub_label = parts[1];
        let sub_type = parts[0];

        // Ensure the second part is "_sub"
        if sub_label != "_sub" {
            return Err(MdnsError::InvalidSublabel);
        }

        return check_mdns_label(sub_type, true);
    }

    Ok(MdnsLabelType::ServiceType)
}

#[cfg(test)]
mod local_network_state_tests {
    use super::LocalNetworkState;

    #[test]
    fn test_parse_known_states_round_trip() {
        assert_eq!(
            LocalNetworkState::parse("granted"),
            LocalNetworkState::Granted
        );
        assert_eq!(
            LocalNetworkState::parse("denied"),
            LocalNetworkState::Denied
        );
        assert_eq!(
            LocalNetworkState::parse("prompt"),
            LocalNetworkState::Prompt
        );
        assert_eq!(
            LocalNetworkState::parse("prompt-with-rationale"),
            LocalNetworkState::PromptWithRationale
        );
    }

    #[test]
    fn test_parse_unknown_state_denies() {
        assert_eq!(
            LocalNetworkState::parse("granted-ish"),
            LocalNetworkState::Denied
        );
        assert_eq!(LocalNetworkState::parse(""), LocalNetworkState::Denied);
    }

    #[test]
    fn test_only_granted_is_granted() {
        assert!(LocalNetworkState::Granted.is_granted());
        assert!(!LocalNetworkState::Denied.is_granted());
        assert!(!LocalNetworkState::Prompt.is_granted());
        assert!(!LocalNetworkState::PromptWithRationale.is_granted());
    }
}

#[cfg(test)]
mod service_type_tests {
    use super::{check_service_type_fully_qualified, MdnsError, MdnsLabelType};

    #[test]
    fn test_valid_service_type_is_accepted() {
        assert_eq!(
            check_service_type_fully_qualified("_http._tcp.local."),
            Ok(MdnsLabelType::ServiceType)
        );
    }

    #[test]
    fn test_valid_subtype_is_accepted() {
        assert_eq!(
            check_service_type_fully_qualified("_printer._sub._http._tcp.local."),
            Ok(MdnsLabelType::SubType)
        );
    }

    #[test]
    fn test_missing_trailing_dot_is_rejected() {
        assert_eq!(
            check_service_type_fully_qualified("_http._tcp.local"),
            Err(MdnsError::MissingTrailingDot)
        );
    }

    #[test]
    fn test_wrong_domain_is_rejected() {
        assert_eq!(
            check_service_type_fully_qualified("_http._tcp.example."),
            Err(MdnsError::InvalidDomain)
        );
    }
}

#[cfg(test)]
mod escaping_tests {
    use super::bytes_option_to_string_option_with_escaping;

    #[test]
    fn test_valid_utf8_passes_through() {
        assert_eq!(
            bytes_option_to_string_option_with_escaping(Some(b"hello".as_slice())),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_control_characters_are_escaped() {
        assert_eq!(
            bytes_option_to_string_option_with_escaping(Some(b"a\x01b".as_slice())),
            Some("a\\u0001b".to_string())
        );
    }

    #[test]
    fn test_invalid_utf8_is_hexlified() {
        assert_eq!(
            bytes_option_to_string_option_with_escaping(Some([0xff, 0x00].as_slice())),
            Some("ff00".to_string())
        );
    }

    #[test]
    fn test_none_stays_none() {
        assert_eq!(bytes_option_to_string_option_with_escaping(None), None);
    }
}
