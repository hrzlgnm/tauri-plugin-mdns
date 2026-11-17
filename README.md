# tauri-plugin-mdns

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-mdns)](https://crates.io/crates/tauri-plugin-mdns)
[![npm](https://img.shields.io/npm/v/tauri-plugin-mdns-api)](https://www.npmjs.com/package/tauri-plugin-mdns-api)
[![License: MIT-0](https://img.shields.io/badge/License-MIT--0-blue.svg)](https://opensource.org/license/mit-0)

mDNS service discovery (browse only) for Tauri apps on desktop and
Android, built on [`mdns-sd`](https://docs.rs/mdns-sd). It finds service
types, resolves instances, reports removals, and exposes interface
selection, protocol flags, daemon metrics and status, and instance
verification.

iOS is not supported and won't be; iOS builds fail with an explicit
error.

## Android

Android 17 blocks local-network access, mDNS included, for apps
targeting SDK 37+ until the user grants `ACCESS_LOCAL_NETWORK`. The
plugin declares the permission in its own manifest, so it survives
`tauri android init`. Browsing refuses to start until access is
granted; request it from the frontend (see below). Below Android 17,
access is always granted.

## Quick Start

In `src-tauri`:

```sh
cargo add tauri-plugin-mdns
```

Register the plugin. It manages its own daemon and state, so no
`.manage()` is needed:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_mdns::init())
```

Grant its default permission in your app's capabilities:

```json
{
    "permissions": [
        "mdns:default"
    ]
}
```

## Commands and events

Commands, under `plugin:mdns|`: `browse_types`, `browse_many`,
`stop_browse`, `verify`, `subscribe_interfaces`, `set_interfaces`,
`get_protocol_flags`, `set_protocol_flags`, `subscribe_metrics`,
`subscribe_daemon_status`, `local_network_status`,
`request_local_network_access`.

Events, emitted to the calling window: `service-type-found`,
`service-resolved`, `service-removed`, `interfaces-changed`,
`metrics-changed`, `daemon-status-changed`.

## JavaScript API

[`tauri-plugin-mdns-api`](https://www.npmjs.com/package/tauri-plugin-mdns-api)
wraps the commands and events:

```sh
npm add tauri-plugin-mdns-api
```

```ts
import {
  browseMany,
  browseTypes,
  localNetworkStatus,
  onServiceResolved,
  onServiceTypeFound,
  requestLocalNetworkAccess,
} from 'tauri-plugin-mdns-api';

if ((await localNetworkStatus()) !== 'granted') {
  // Explain why first, and request from a user gesture. While denied,
  // show a blocking empty state pointing to
  // Settings → Apps → Permissions → Nearby devices.
  await requestLocalNetworkAccess();
}

await onServiceTypeFound(({ service_type }) => {
  void browseMany([service_type]);
});
await onServiceResolved(({ service }) => {
  console.log(service.instance_fullname);
});
await browseTypes();
```

## License

MIT-0
