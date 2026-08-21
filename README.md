# Mortgage Calculator

A mortgage payment, amortization, affordability, and refinance calculator,
with a pure Rust calculation core shared by two separate UIs: a React web
app that loads the core compiled to WebAssembly, and a [Slint](https://slint.dev)
app that runs the same Rust code natively on desktop, iOS, and Android (and
also compiles to a wasm canvas for the browser).

## Features

- **Payment**: standard amortizing payment amount, total cost, total interest
- **Amortization**: full payment-by-payment schedule with optional extra
  principal payments, and the resulting time/interest saved
- **Affordability**: maximum home price from income, debts, down payment,
  and a target debt-to-income ratio (property tax, insurance, and HOA aware)
- **Refinance**: break-even point and lifetime savings from refinancing into
  a new rate/term, net of closing costs
- **Compare**: side-by-side scenario comparison across rate types (fixed, or
  floating expressed as base index + spread, e.g. "SOFR + 2.5%") and loan
  terms, with a handful of common presets plus fully custom entries
- **Saved scenarios**: every calculator (including Compare) can save its
  current inputs under a name and reload them later — persisted locally in
  the browser via a real embedded database (see below), not just this
  session's memory

All calculations run **entirely client-side** — the WASM module runs in the
browser, no server round-trip, and saved data never leaves the device.

## Architecture

```
crates/
  mortgage-core     - shared vocabulary: PaymentFrequency, rounding, errors
  mortgage-calc     - pure calculation logic (Loan + payment/amortization/
                      affordability/refinance/comparison modules, RateType);
                      no I/O, no JS
  mortgage-ports    - the Scenario type + ScenarioStore trait; no backend
  mortgage-ext-redb - ScenarioStore implemented on redb: a plain file on
                      native targets, an in-memory buffer asynchronously
                      flushed to IndexedDB in the browser
  mortgage-wasm     - wasm-bindgen bindings: parses JsValue, calls
                      mortgage-calc / mortgage-ext-redb, serializes the
                      result back. No business logic lives here.
  mortgage-ui-slint - Slint UI: one Rust codebase targeting desktop, iOS,
                      Android, and a wasm canvas for the browser
www/                - webpack + React app that loads the compiled wasm module
```

The calculation crates are 100% pure Rust and unit-tested on their own, with
no knowledge of `wasm-bindgen`, Slint, or any other UI layer. That keeps the
math reusable across both UIs (and any future one) without dragging their
dependencies along, and keeps each UI crate itself trivial to review since
it's pure plumbing over `mortgage-calc`.

Money is `rust_decimal::Decimal` throughout the Rust layers (never `f64`)
to avoid floating-point drift in currency arithmetic; the wasm boundary
converts to/from `f64` since that's what JS numbers are.

### Local persistence: redb, not SQLite

Saved scenarios are stored with [`redb`](https://docs.rs/redb), a pure-Rust
embedded database, rather than SQLite. `mortgage-ext-redb::native` opens a
plain redb file on disk, used by the Slint app's desktop, iOS, and Android
builds.

The browser path (`mortgage-ext-redb::wasm`) is the interesting one: redb's
pluggable `StorageBackend` trait is implemented over an in-memory buffer,
with each write asynchronously flushed to IndexedDB in the background
(via the `rexie` crate). Real redb — actual ACID transactions, actual
schema — runs in-process on every read/write; only the durability step is
async and best-effort. This deliberately avoids the Origin Private File
System's synchronous-access-handle API, which is the "proper" way to get
a real file-backed database in a browser but only works inside a
dedicated Worker — using it would have meant restructuring the whole app
around a Worker/postMessage boundary just for storage. The tradeoff: a
write is durable within roughly one JS microtask rather than
fsync-before-return, which is a normal local-first compromise.

## Quick start

```bash
# Run the Rust test suite
cargo test --workspace

# Build the wasm bindings, then run the dev server
cd www
npm install
npm run build:wasm
npm start          # http://localhost:3001

# Production build (runs build:wasm first)
npm run deploy
```

`npm start` also works without `build:wasm` having been run yet — `src/index.js`
falls back to a mock JS implementation of the same functions so the UI is
usable before you've built the wasm module.

### Slint app (desktop / web / iOS / Android)

```bash
# Desktop (native window)
cargo run -p mortgage-ui-slint

# macOS: package it as a .app so it gets the real Dock/Finder icon.
# macOS reads the icon from the bundle, ignoring Slint's `Window.icon`, so
# a bare `cargo run` always shows a generic executable icon. The bundle is
# unsigned -- right-click -> Open to bypass Gatekeeper locally.
./scripts/bundle-macos.sh            # add --debug to skip the release build
open "target/macos/Mortgage Calculator.app"

# Web (compiles to a wasm canvas, separate from the React app above)
cd crates/mortgage-ui-slint
wasm-pack build --target web --out-dir pkg
python3 -m http.server 4173   # then open index.html

# iOS (generates an Xcode project that cargo-builds the Rust binary
# as a postCompileScript; open in Xcode or `xcodebuild` from there)
cd crates/mortgage-ui-slint/ios
xcodegen generate

# Android (via `xbuild` — https://github.com/rust-mobile/xbuild — which
# cross-compiles and packages the slint/backend-android-activity binary
# into an APK; package identity comes from crates/mortgage-ui-slint/manifest.yaml).
# Needs the Android SDK + NDK and `cargo install --git https://github.com/rust-mobile/xbuild.git xbuild`.
export ANDROID_HOME=/path/to/android-sdk       # must contain ndk/<version>/
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/<version>"
# The system clang (Xcode's, on macOS) resolves `-fuse-ld=lld` by searching
# PATH, not by asking the NDK — without this, linking fails with
# "invalid linker name in argument '-fuse-ld=lld'" even though the NDK
# ships its own copy.
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH"
x build --manifest-path crates/mortgage-ui-slint/Cargo.toml --platform android --arch arm64 --format apk
# `x run` instead of `build` installs + launches on a connected device/emulator.
```

## App icon

Every platform's icon is generated from one script, so the artwork has a
single source of truth:

```bash
python3 assets/icon/generate.py    # needs Pillow; .icns step needs macOS
```

It writes the iOS asset catalog, the Android launcher PNG, the Slint window
icon, `.icns`/`.ico` for desktop packaging, and the web favicons plus PWA
manifest icons. All of its output is committed — rerun it only when the
artwork itself changes.

The mark is a house with a mortgage's remaining-balance curve carved through
it. The curve is the real B(t) for a level-payment loan rather than a
decorative swoosh, which is why it stays high through the first half of the
term and only breaks late.

Two platform quirks the script handles, both easy to get wrong by hand:

- **iOS** rejects App Store icons that carry an alpha channel, so the 1024px
  entry is written as opaque RGB.
- **Android** icons here go through `xbuild`, which scales a single PNG into
  the legacy mipmap densities and does *not* emit an adaptive icon with
  separate foreground/background layers. Launchers therefore apply their own
  mask to the square, so that variant is rendered with a safe-zone inset —
  without it a circular mask clips the eaves of the house.
- **macOS** takes the icon from the `.app` bundle and ignores the window
  icon entirely, so the generated `.icns` only shows up once the binary is
  bundled — see `scripts/bundle-macos.sh` above.

## A note on the `--target web` + webpack combination

`mortgage-wasm` is built with `wasm-pack build --target web`, whose
generated glue does its own `fetch` + `WebAssembly.instantiate` of the
`.wasm` binary. Webpack's native `experiments.asyncWebAssembly` /
`syncWebAssembly` **must stay disabled** for that combination — turning
them on lets webpack intercept the glue's internal wasm import and hand
back a differently-linked module, which silently corrupted values crossing
the JS/wasm boundary for some (not all) exported functions during
development of this project. See the comment in `www/webpack.config.js`.
