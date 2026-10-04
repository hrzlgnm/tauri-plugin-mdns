# Changelog

## [0.3.2](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.3.1...tauri-plugin-mdns-v0.3.2) (2026-10-04)


### Bug Fixes

* raise Android compileSdk to 37 for androidx.core 1.19 ([#43](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/43)) ([f7b350b](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/f7b350b8aba960724619d3e17b69b47474a8017d))


### Miscellaneous Chores

* check release AAR metadata in Android CI ([#45](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/45)) ([7813fa7](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/7813fa70b22c6d1f1f5dced10d33180d7e74c984))

## [0.3.1](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.3.0...tauri-plugin-mdns-v0.3.1) (2026-10-04)


### Bug Fixes

* allow skips in the CI Status gate ([#38](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/38)) ([6268538](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/6268538fc811b9be3e77d344314bc778d332e3be))
* declare google and mavenCentral repositories for Android library ([#27](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/27)) ([49e5839](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/49e5839122fc5e83147c956713ba5ccbc1dec767))
* migrate Android jvmTarget to compilerOptions DSL ([#40](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/40)) ([255c9d6](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/255c9d65a8253717767e97514df042ace2d20262))
* trigger patch releases for chore and refactor commits ([#25](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/25)) ([5946e4d](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/5946e4d8b9ebd1d759dbb3b4c4534ad3495494e7))


### Dependencies

* update actions/setup-java action to v6 ([0e29c92](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/0e29c92300ff0b9761e93db34be3e2a4c4f383ed))
* update dependency androidx.appcompat:appcompat to v1.8.0 ([c511fcb](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/c511fcba372268c4aa600a8769f867cb11570bb9))
* update dependency androidx.core:core-ktx to v1.19.1 ([04c9188](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/04c9188a4aa96993b38b7b75bf8f5f67696205fb))
* update dependency androidx.test.espresso:espresso-core to v3.7.0 ([2e155c3](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/2e155c3b733d21c6095e898bbd54b532ea25f334))
* update dependency androidx.test.ext:junit to v1.3.0 ([1b2e594](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/1b2e594e1afbf8131a13c81d7dca30d2aad550cf))
* update dependency com.google.android.material:material to v1.14.0 ([dea921e](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/dea921e700978e12262b3c8cc9c6a5ed05bf4b21))
* update dependency java-jdk to v25 ([943c139](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/943c1392bc9c3f76e353f54a92d10149814af11f))


### Miscellaneous Chores

* cover Android Kotlin via example debug APK build in CI ([#30](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/30)) ([a815d6c](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/a815d6c77a5bc3acb913837252633eed07175cc1))
* emit bare deps prefix for dependency updates ([#41](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/41)) ([6670b66](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/6670b6642d107705c65afd71b5446c4db5cd5ef9))
* enable sccache for the APK build ([#37](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/37)) ([089c2f7](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/089c2f790f8e0ba5506ff41828164117f4253c36))
* **renovate:** remove tagging deps as fix rule ([#33](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/33)) ([c46d225](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/c46d225830fcb20a1d0263b443e61ca771ce11b6))
* run CI jobs only on related changes ([#36](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/36)) ([d1020b1](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/d1020b1f13d3fafdd1dac384c8c2b832544aea18))


### Code Refactoring

* drop the iOS fallback; mobile is Android-only ([#24](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/24)) ([48aecfe](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/48aecfe61ba746e37bd6e3fbbfa6180386b67ff1))


### Continuous Integration

* cover the Android (mobile) runtime path ([#23](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/23)) ([e00ccc2](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/e00ccc26115925d1046bd6c58bf78f3c61b9fd67))

## [0.3.0](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.2.5...tauri-plugin-mdns-v0.3.0) (2026-10-04)


### Features

* export ServiceTypes from the JavaScript API ([#20](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/20)) ([9a18edb](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/9a18edb8cedab6f21d4f7aa724b34c1e560d3ad6))

## [0.2.5](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.2.4...tauri-plugin-mdns-v0.2.5) (2026-10-04)


### Bug Fixes

* drop wrong PermissionState import in Android plugin class ([#16](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/16)) ([c13476c](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/c13476c15217ed0ce7e072efd35014692b7c62f3))

## [0.2.4](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.2.3...tauri-plugin-mdns-v0.2.4) (2026-10-04)


### Bug Fixes

* import PermissionState in Android plugin class ([#14](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/14)) ([5f895f2](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/5f895f2acb83f000815f00228f6aedc6a7e5952b))

## [0.2.3](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.2.2...tauri-plugin-mdns-v0.2.3) (2026-10-04)


### Bug Fixes

* declare android project path in plugin build ([#12](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/12)) ([6fdbd3e](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/6fdbd3e6928d8bbb7456527062f75515fe874ca4))

## [0.2.2](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.2.1...tauri-plugin-mdns-v0.2.2) (2026-10-04)


### Bug Fixes

* use explicit closure for permission state parsing ([#10](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/10)) ([1649d55](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/1649d55313e7001c5ed5d67bd9ac4dad79c5ce30))

## [0.2.1](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.2.0...tauri-plugin-mdns-v0.2.1) (2026-10-04)


### Bug Fixes

* install dependencies before npm dry-run publish ([#2](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/2)) ([c3ef096](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/c3ef09674668f33884614898c915265cf6db4241))
* stage npm publishes instead of publishing directly ([#4](https://github.com/hrzlgnm/tauri-plugin-mdns/issues/4)) ([351fdc6](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/351fdc6c1cb0a2463fd0ce85bd1c81c29285245c))

## [0.2.0](https://github.com/hrzlgnm/tauri-plugin-mdns/compare/tauri-plugin-mdns-v0.1.0...tauri-plugin-mdns-v0.2.0) (2026-10-04)


### Features

* add tauri-plugin-mdns repository ([f0af874](https://github.com/hrzlgnm/tauri-plugin-mdns/commit/f0af87416ec4faf8e1f85ee6e4b717ed4045b8b4))
