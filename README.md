# tauri-plugin-mdns

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-mdns)](https://crates.io/crates/tauri-plugin-mdns)
[![npm](https://img.shields.io/npm/v/tauri-plugin-mdns-api)](https://www.npmjs.com/package/tauri-plugin-mdns-api)
[![License: MIT-0](https://img.shields.io/badge/License-MIT--0-blue.svg)](https://opensource.org/license/mit-0)

mDNS service discovery (browse) for Tauri apps on desktop and Android,
backed by the [`mdns-sd`](https://docs.rs/mdns-sd) crate in Rust on
every supported platform: service-type enumeration, instance browsing
with resolution and removal events, interface selection with protocol
flags, daemon metrics, and instance verification.

Mobile support is Android-only: iOS is not supported and there are no
plans to port the plugin to it (iOS builds fail with an explicit
error).

On Android the plugin additionally handles the `ACCESS_LOCAL_NETWORK`
runtime permission, which Android 17 mandates for apps targeting SDK
37+: without it local-network access — including mDNS — is blocked by
default, so browsing refuses to start while access is not granted. The
permission declaration travels in the plugin manifest (merged into the
host app, surviving `tauri android init`); the consent flow is driven
from the frontend (see below).

## Quick Start

Add the plugin to your `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri-plugin-mdns = "0.1"
```

Register it and grant the plugin's `default` permission in your app's
capabilities so the frontend can invoke its commands:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_mdns::init())
```

```json
{
    "permissions": [
        "mdns:default"
    ]
}
```

## Commands

The plugin registers its commands under the `plugin:mdns|` namespace:
`browse_types`, `browse_many`, `stop_browse`, `verify`,
`subscribe_interfaces`, `set_interfaces`, `get_protocol_flags`,
`set_protocol_flags`, `subscribe_metrics`, `local_network_status`,
and `request_local_network_access`. Discovery results arrive as window
events: `service-type-found`, `service-resolved`, `service-removed`,
`interfaces-changed`, and `metrics-changed`.

## JavaScript API

The
[`tauri-plugin-mdns-api`](https://www.npmjs.com/package/tauri-plugin-mdns-api)
package wraps these commands and events for JavaScript frontends:

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
  stopBrowse,
} from 'tauri-plugin-mdns-api';

const status = await localNetworkStatus();
if (status !== 'granted') {
  // Show a rationale, then request from a user gesture. While denied,
  // gate discovery on a blocking empty state with guidance to
  // Settings → Apps → Permissions → Nearby devices.
  await requestLocalNetworkAccess();
}

const unlisten = await onServiceTypeFound(({ service_type }) => {
  void browseMany([service_type]);
});
await browseTypes();
```

## How it works

The plugin manages the shared `mdns-sd` daemon and all browsing state
itself, so no additional `.manage()` is required. The browsing commands
check local-network access before starting (Android 17+ targeting SDK
37); below Android 17 access is implicitly granted and the check
short-circuits to granted without prompting.

## License

MIT-0
