# mortgage-ui

One Rust codebase, five calculator screens (Payment, Amortization,
Affordability, Refinance, Compare), targeting desktop, iOS, Android, and
web from [Dioxus](https://dioxuslabs.com). Calls `mortgage-calc` and
`mortgage-ports`/`mortgage-ext-redb` directly — no JSON/DTO boundary, no
`wasm-bindgen`, unlike `mortgage-wasm` + `www/`.

## Running

```bash
dx serve --desktop   # fastest feedback loop; no simulator/browser needed
dx serve --web        # http://localhost:8080
dx serve --ios         # requires Xcode + a booted iOS Simulator
dx serve --android    # experimental — see Dioxus's own docs on setup
```

`--desktop`/`--web`/`--ios`/`--android` each select a different
`dioxus` renderer feature (see the `[features]` table in `Cargo.toml`) —
`desktop` and `mobile` both compile to the same underlying wry/tao
renderer crate, just packaged differently by `dx`.

## Storage

Scenarios are saved via `mortgage-ext-redb::RedbScenarioStore` — the same
type `mortgage-wasm` uses, opened differently per platform (see
`src/storage.rs`):

- Native (desktop/iOS/Android): a plain redb file in
  `~/Library/Application Support/MortgageCalculator/` (works today on
  Apple platforms; Android needs its own data-directory convention before
  scenarios will persist there).
- Web: the same in-memory-buffer-flushed-to-IndexedDB backend
  `mortgage-wasm` uses.

`App` shows a brief loading screen while the store opens (instant on
native, awaits one IndexedDB round trip on web) before mounting the rest
of the UI.
