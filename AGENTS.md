# Agent conventions

This is a Tauri v2 plugin crate with TypeScript bindings
(`tauri-plugin-mdns` on crates.io, `tauri-plugin-mdns-api` on npm).
It provides mDNS service discovery (browse) for Tauri apps on desktop
and mobile, backed by the `mdns-sd` crate in Rust on every platform.
On Android it additionally handles the `ACCESS_LOCAL_NETWORK` runtime
permission, which Android 17 mandates for apps targeting SDK 37+ and
without which local-network access — including mDNS — is blocked by
default.

## Workflow

- Work on a typed branch (`feat/...`, `fix/...`, `chore/...`, etc.)
  and land changes through a pull request; direct pushes to `main`
  are blocked.
- Treat every task as authorizing commits and a pull request unless
  the user opts out. Commit each complete logical unit as soon as
  its applicable checks pass.
- Refine work already represented by a commit with
  `git commit --fixup=<sha>`, including when the target is `HEAD`.
  Never amend, autosquash, or fold fixups; the user does that.

## Validation

- Rust: `cargo fmt -- --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo metadata --locked`.
- JavaScript: `npm ci`, `npm run build` (rollup + `tsc`, Node 24).
- Workflows: `actionlint .github/workflows/*.yml`.
- Text: `typos` (config in `typos.toml`).
- Renovate config: `npx --yes -p renovate@latest renovate-config-validator .github/renovate.json5`.
- Every commit must compile, pass its tests, and be format- and
  lint-clean. No `unsafe` code, no `#[allow(warnings)]`.

## CI system dependencies

`clippy`, `test`, and crate publishing compile the full `tauri`
dependency stack, whose Linux build scripts (`webkit2gtk-sys`,
`javascriptcore-rs-sys`, `soup3-sys` via the default `wry` feature)
require `libwebkit2gtk-4.1-dev` on any Linux host — even though the
plugin only executes its native code on Android. `libappindicator3-dev`,
`librsvg2-dev`, and `patchelf` are deliberately absent:
`libappindicator-sys` and `librsvg-sys` are not in the built dependency
graph (`cargo tree -i <sys-crate>` proves it; the lockfile still lists
`libappindicator-sys` via the optional, unactivated `tray-icon`
dependency) and `patchelf` is only invoked by
tauri-bundler, which plugin CI never runs. Trimming `tauri` default
features would drop `webkit2gtk` but not the non-optional `gtk`/`muda`
dependencies, so it swaps one system dependency for another without
removing the requirement — keep default features. Do not re-add the
dropped packages without proving a `-sys` crate re-entered the graph.

## Android native code

- The plugin ships its own `android/` library module (manifest +
  Kotlin), merged into the host app by Gradle's manifest merger, so
  the `ACCESS_LOCAL_NETWORK` declaration survives
  `tauri android init`. Never rely on hand-edits to the app's
  `gen/android`.
- The `MdnsPlugin` Kotlin class carries the
  `@TauriPlugin(permissions = ...)` annotation with the
  `localNetwork` alias; the base `Plugin` class auto-implements
  `checkPermissions`/`requestPermissions` from it. The overrides of
  both must NOT re-annotate `@Command`: the base registrations plus
  virtual dispatch already route to the overrides, and re-annotating
  registers the base method entry last, shadowing them in the command
  map.
- `checkPermissions` and `requestPermissions` resolve `GRANTED` below
  Android 17 (API 37), where the permission does not exist and access
  is implicitly granted; requesting an unknown permission there would
  misreport.
- The library keeps `compileSdk = 37` (required by androidx.core 1.19+)
  and refers to the permission by string literal, so it compiles without
  referencing the API-37-only constant. `consumer-rules.pro` keeps the
  reflectively-loaded plugin class across the host app's release
  minification.
- No `ios/` directory and no iOS support by design: the mobile
  engine is Android-only and iOS builds fail with an explicit
  `compile_error!` in `src/mobile.rs`. Never add iOS shims,
  fallbacks, or CI targets.

## Releases

- `CHANGELOG.md`, versions, and `v`-less `tauri-plugin-mdns-vMAJOR.MINOR.PATCH`
  tags are owned by release-please (`release-please-config.json` +
  `.release-please-manifest.json`); never edit the changelog or push
  version tags manually. The Rust crate version and the npm package
  version always move in lockstep via `extra-files`.
- Publishing uses trusted publishing (GitHub OIDC), never
  long-lived tokens: `rust-lang/crates-io-auth-action` for
  crates.io, `npm stage publish` (approved later by a maintainer) for npm. Both publishers
  must be registered before the first publish; the workflows fail
  otherwise.

## Commits

- Use conventional commit subjects (`feat:`, `fix:`, `chore:`,
  etc.). Explain why in the message rather than paraphrasing the
  diff. Wrap body lines at 72 characters.
- Add both trailers to every commit with `--trailer`; never use
  `--author` or `--committer` for attribution:

  ```text
  Co-authored-by: opencode <noreply@opencode.ai>
  Assisted-by: opencode (<model-name>)
  ```

## Code Review

Mandatory gate: after validation passes on the final commit(s),
run a two-axis review (repo standards incl. this file versus the
originating request) and fix its findings before pushing or
opening a PR.
Keep pull request descriptions to summary and issue references;
omit testing recaps, CI and Validation already cover those.
