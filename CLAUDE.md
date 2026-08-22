# Working in this repo

## The rule: business logic lives in Rust

Every calculation, rule, threshold and regulatory constant belongs in the Rust
crates. The front ends render results and collect input. They do not compute.

This is not a style preference. There are four front ends over one core — web
(wasm), desktop, iOS, Android — and a rule implemented twice is a rule that
will eventually disagree with itself. When it does, the app does not crash: it
shows a confident, wrong number. A mortgage calculator that is quietly wrong is
worse than one that is visibly broken.

### Where a thing goes

| Layer | Owns |
|---|---|
| `mortgage-core` | Shared vocabulary: `Region`, `PaymentFrequency`, rounding, errors |
| `mortgage-calc` | Every calculation and every regulatory rule (MAS, IRAS, CPF, FHFA) |
| `mortgage-wasm` | Bridge only. Parse `JsValue`, call `mortgage-calc`, serialize back |
| `www/`, `mortgage-ui-slint` | Layout, input, formatting for display, i18n |

### Business logic (Rust) vs. host layer (front end)

Business logic is anything where a second implementation could give a
different answer:

- arithmetic on money, rates, terms, or dates
- thresholds, tiers, ceilings, floors, eligibility
- deriving one domain value from another (loan from price and deposit,
  years from payment periods, a percentage of a price)
- choosing between rulesets, and ranking the signals that drive that choice

Host layer is anything a wasm module cannot reach or that has no domain
content:

- reading `navigator`, `Intl`, `localStorage`, the URL
- DOM, layout, SVG geometry, chart coordinates, downsampling for rendering
- number and date *formatting* for display, separators, catalog lookup

`www/src/region.js` is the model to copy: it reads the browser signals Rust
cannot see, hands them over, and makes no decision itself.

### Adding a calculation

1. Write it in `mortgage-calc`, with tests, and cite the authority in a doc
   comment when it encodes a regulatory constant.
2. Add a binding in `mortgage-wasm` that only parses, calls and serializes.
   The `bridge_coverage` test fails if a public `mortgage-calc` module has no
   binding.
3. Call it from the front end.

If you catch yourself writing arithmetic in `.jsx`, stop -- that is the
signal, and it is nearly always a sign the Rust side is missing an export
rather than a sign JavaScript is the right place.

## Known violations, not yet fixed

Named so they are not mistaken for precedent:

- `www/src/components/RefinanceCalculator.jsx` -- extra periods from term
- `www/src/components/Charts.jsx` -- interest share of total paid
- `www/src/duration.js` -- payment periods to years and months
- `www/src/index.js` -- `createMockModule` is a **full parallel
  implementation** of payment, amortization, affordability, refinance and
  comparison in JavaScript. It already declines to reimplement the Singapore
  rules, the scenario figures and region detection, and says why; the rest
  should follow. `npm start` now builds the wasm first, so this fallback only
  runs on a machine without wasm-pack installed.

## Verification traps

Each of these produces a wrong result that looks like a correct one.

- **`npm run build` does not rebuild the wasm.** It is webpack-only. Run
  `npm run build:wasm` first, or you are testing the previous `pkg/`.
- **`cargo check --workspace` misses wasm32-only breakage.** Code behind
  target gates compiles for the host and fails for wasm32. Run
  `cargo build -p mortgage-wasm --target wasm32-unknown-unknown`.
- **jsdom here has no `localStorage`**, on `window` or as a bare global. Every
  storage path in the app runs into its catch block under test unless the test
  stands up a fake. See `www/src/region.test.js`.
- **`wasm-opt` is off deliberately.** The reason is measured and recorded in
  `crates/mortgage-wasm/Cargo.toml`. Do not "fix" it.
- **Never round-trip the i18n catalogs through Python's `unicode_escape`.** It
  decodes UTF-8 as Latin-1 and turns every CJK character into mojibake. A test
  guards this (no catalog may contain U+0080-U+00FF); write files with
  `encoding='utf-8'` instead.

## Landing changes

One branch per round of work, focused commits, then a PR with a Summary and
Test plan. **Do not self-merge** -- branch protection requires approval. Wait
for the merge, then verify `state == "MERGED"` in its own step before deleting
any branch; deleting the head branch of an open PR closes it.
