# Mortgage Calculator

A mortgage payment, amortization, affordability, and refinance calculator,
built as a **Rust backend compiled to WebAssembly** with a React frontend —
the same architecture as the [`convex`](../convex) fixed-income library:
pure calculation logic in one layer, a thin `wasm-bindgen` crate translating
JS &lt;-&gt; Rust in another, and a webpack app that dynamically loads the
compiled `.wasm` module in the browser.

## Features

- **Payment**: standard amortizing payment amount, total cost, total interest
- **Amortization**: full payment-by-payment schedule with optional extra
  principal payments, and the resulting time/interest saved
- **Affordability**: maximum home price from income, debts, down payment,
  and a target debt-to-income ratio (property tax, insurance, and HOA aware)
- **Refinance**: break-even point and lifetime savings from refinancing into
  a new rate/term, net of closing costs

All calculations run **entirely client-side** — the WASM module runs in the
browser, no server round-trip.

## Architecture

```
crates/
  mortgage-core   - shared vocabulary: PaymentFrequency, rounding, errors
  mortgage-calc   - pure calculation logic (Loan + payment/amortization/
                    affordability/refinance modules); no I/O, no JS
  mortgage-wasm   - wasm-bindgen bindings: parses JsValue, calls mortgage-calc,
                    serializes the result back. No business logic lives here.
www/              - webpack + React app that loads the compiled wasm module
```

This mirrors `convex`'s layering (`convex-core` -> `convex-bonds` /
`convex-analytics` -> `convex-wasm` -> `www`): the calculation crates are
100% pure Rust and unit-tested on their own, and only the top `-wasm` crate
knows about `wasm-bindgen`. That keeps the math reusable from a future CLI
or server without dragging wasm-bindgen along, and keeps the wasm crate
itself trivial to review since it's pure plumbing.

Money is `rust_decimal::Decimal` throughout the Rust layers (never `f64`)
to avoid floating-point drift in currency arithmetic; the wasm boundary
converts to/from `f64` since that's what JS numbers are.

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
usable before you've built the wasm module, mirroring `convex/www`'s dev
fallback.

## A note on the `--target web` + webpack combination

`mortgage-wasm` is built with `wasm-pack build --target web`, whose
generated glue does its own `fetch` + `WebAssembly.instantiate` of the
`.wasm` binary. Webpack's native `experiments.asyncWebAssembly` /
`syncWebAssembly` **must stay disabled** for that combination — turning
them on lets webpack intercept the glue's internal wasm import and hand
back a differently-linked module, which silently corrupted values crossing
the JS/wasm boundary for some (not all) exported functions during
development of this project. See the comment in `www/webpack.config.js`.
