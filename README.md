# Mortgage Calculator

A mortgage payment, amortization, affordability, and refinance calculator for
the **US and Singapore markets**, built as a web app with a pure Rust
calculation core compiled to WebAssembly.

**<https://dengf.github.io/mortgage_calculator/>** — nothing to install, and
nothing you type is sent anywhere.

## What it is, and what it deliberately is not

This is a web app and only a web app. There used to be a [Slint](https://slint.dev)
crate here that built the same Rust core into a desktop, iOS and Android app;
it was removed on purpose. A calculator someone consults a handful of times
while deciding on a house is not something they want to install, keep updated,
or wait behind a store review for. One URL, one build, one place a fix has to
land.

Three consequences, all intentional:

- **No installation.** It is a page. It works on a phone, and the browser's
  own "Add to Home Screen" is there for anyone who wants a launcher — that is
  the user's choice, not a prerequisite.
- **No accounts, no server, no data collection.** There is no backend to send
  anything to: the site is static files on GitHub Pages. The app makes no
  analytics, telemetry or tracking calls of any kind. Open the network tab and
  watch it stay empty.
- **Saved scenarios stay on the device.** They are written to IndexedDB in
  your own browser (see below). Nothing is uploaded, and clearing site data
  removes them for good.

## Features

- **Payment**: the amortizing instalment, total cost, and total interest
- **Amortization**: the full payment-by-payment schedule with optional extra
  principal, and the time and interest that saves
- **Affordability**: maximum home price from income, debts, and deposit — for
  the US against a target debt-to-income ratio, for Singapore against MAS's
  TDSR and MSR limits, LTV ceilings, CPF usage and IRAS stamp duty
- **Refinance**: break-even point and lifetime savings net of closing costs
- **Compare**: scenarios side by side across rate shapes and terms
- **Report**: a printable, client-facing illustration — payment bands, a
  rate-rise stress table, the schedule at either granularity, a watermark and
  a cited source list. Print to PDF or open a pre-filled mail draft
- **Saved scenarios**: name the current inputs and reload them later
- **Three languages**: English, 简体中文, 繁體中文

### Rate shapes

A rate is entered as a *shape*, not a number, on every tab:

- **Fixed** — one rate for the term
- **Floating** — a published benchmark plus a spread, e.g. `SOFR + 2.5%`
- **Steps up** — a promotional spread for a lock-in period and a higher one
  thereafter. This is how every Singapore bank quotes a home loan, and reading
  only the promotional rate is the mistake the structure invites. MAS Notice
  645 assesses servicing on the *thereafter* rate, and so does this.

Where a rate rests on a published benchmark the app cannot read, it says so on
screen and on the printed document rather than presenting a projection as a
quotation.

## Architecture

```
crates/
  mortgage-core     - shared vocabulary: PaymentFrequency, rounding, errors
  mortgage-calc     - all calculation and all domain rules (Loan, RateType,
                      payment/amortization/affordability/refinance/comparison,
                      the US and Singapore rule sets, and what a report says).
                      No I/O, no JS.
  mortgage-ports    - the Scenario type + ScenarioStore trait; no backend
  mortgage-ext-redb - ScenarioStore on redb, backed by IndexedDB in the browser
  mortgage-wasm     - wasm-bindgen bindings: parse JsValue, call mortgage-calc,
                      serialize the result back. No business logic lives here.
www/                - webpack + React app that loads the compiled wasm module
```

The direction is one-way: **Rust → wasm → React**. `mortgage-calc` knows
nothing about `wasm-bindgen` and is unit-tested on its own; `mortgage-wasm` is
pure plumbing and is trivial to review because of it.

### Core logic lives in Rust

This is the rule the whole layout exists to serve, and it is not stylistic. The
front end formats and lays out; it does not decide anything. No money
arithmetic, no thresholds, no domain derivations in `.jsx` — a rule
implemented twice is a rule that will be wrong in one of the two places, and
the copy that drifts is the one nobody is testing.

Two consequences worth knowing before adding a feature:

- **Send the shape, not the number.** A domain value with more than one form
  crosses the boundary as that form. `www/src/rate.js` is the model: a quote
  travels as its variant and its fields, and Rust decides what rate that
  charges, what the instalment is before and after a step-up, and what rate a
  bank assesses servicing at. Flattening `base + spread` in JS looks harmless
  and produces a different product.
- **Carry what a figure assumed, next to the figure.** An amount and the
  assumption under it travel together, both decided in Rust. See `CLAUDE.md`.

Money is `rust_decimal::Decimal` throughout the Rust layers, never `f64`, so
currency arithmetic cannot drift. The wasm boundary converts to and from `f64`
because that is what a JS number is.

### Local persistence: redb, not SQLite

Saved scenarios use [`redb`](https://docs.rs/redb), a pure-Rust embedded
database. `mortgage-ext-redb::wasm` implements redb's pluggable
`StorageBackend` over an in-memory buffer, with each write asynchronously
flushed to IndexedDB (via `rexie`) and the whole thing preloaded once at
startup. Real redb — actual transactions, actual schema — runs in-process on
every read and write; only the durability step is async.

This deliberately avoids the Origin Private File System's synchronous access
handles, which are the "proper" route to a file-backed database in a browser
but only exist inside a dedicated Worker. Using them would have meant
restructuring the app around a Worker/postMessage boundary for storage alone.
The tradeoff: a write is durable within about one JS microtask rather than
fsync-before-return, which is a normal local-first compromise.

## Quick start

```bash
# The Rust test suite
cargo test --workspace

# Build the wasm bindings, then run the dev server
cd www
npm install
npm run build:wasm
npm start          # http://localhost:3001

# The front-end test suite
npm test

# Production build (runs build:wasm first)
npm run deploy
```

`npm start` works before `build:wasm` has ever been run — `src/index.js` falls
back to a mock JS implementation so the UI comes up, though the figures are
not the real ones until the wasm module exists.

Two traps that produce a wrong result looking like a correct one:

- **`npm run build` does not rebuild the wasm.** It is webpack-only. Run
  `npm run build:wasm` first, or you are testing the previous `pkg/`.
- **`cargo check --workspace` misses wasm32-only breakage.** Run
  `cargo build -p mortgage-wasm --target wasm32-unknown-unknown`.

## App icon

Every icon comes from one script, so the artwork has a single source of truth:

```bash
python3 assets/icon/generate.py    # needs Pillow
```

It writes the favicons, the PWA manifest icons and the header mark. All of its
output is committed — rerun it only when the artwork itself changes.

The mark is a house with a mortgage's remaining-balance curve carved through
it. The curve is the real B(t) for a level-payment loan rather than a
decorative swoosh, which is why it stays high through the first half of the
term and only breaks late.

## A note on the `--target web` + webpack combination

`mortgage-wasm` is built with `wasm-pack build --target web`, whose generated
glue does its own `fetch` + `WebAssembly.instantiate` of the `.wasm` binary.
Webpack's native `experiments.asyncWebAssembly` / `syncWebAssembly` **must stay
disabled** for that combination — turning them on lets webpack intercept the
glue's internal wasm import and hand back a differently-linked module, which
silently corrupted values crossing the JS/wasm boundary for some (not all)
exported functions during development of this project. See the comment in
`www/webpack.config.js`.

## Not financial advice

This is a calculator and a planning illustration. It is not a Loan Estimate,
not a MAS Residential Property Loan Fact Sheet, and not an offer — those are
regulated disclosures a *lender* issues about a real offer it is making. Every
figure here is an estimate; check it against your bank's own paperwork.

## License

MIT — see [LICENSE](LICENSE).
