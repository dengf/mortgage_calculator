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

## Keeping it that way

The eight JavaScript copies of core rules that this file used to list are all
gone: the scenario figures, the deposit percentage, the amortization grouping,
the comparison verdict, the period-to-time conversion, the interest share, the
region ranking, and the mock module's full second implementation of the
mortgage math.

Nothing in `www/` computes a mortgage figure any more. That is worth checking
before adding one:

- `www/src/index.js` falls back to `createUnavailableModule` when the engine
  cannot load. It **reports** the failure and computes nothing. Do not grow it
  back into a calculator -- a browser that silently gets JavaScript answers is
  worse than one that says it cannot help.
- `www/src/undefined-setters.test.js` guards a different recurring bug: a
  handler calling a setter that no longer exists. It shipped twice.

### Send the shape, not the number

A domain value that can take more than one form crosses the boundary as that
form, never pre-resolved to a figure. `www/src/rate.js` is the model: a quote
travels as `{ kind: "reverting", base_rate_percent, initial_spread_percent,
initial_years, thereafter_spread_percent }`, and Rust decides what rate that
charges, what the instalment is before and after the step-up, and what rate a
bank assesses servicing at.

Flattening it in JS looks harmless -- `base + spread` is one addition -- and
it is the same mistake as any other duplicated rule, with one extra edge: a
flattened quote is not merely rounded differently, it is a *different
product*. Every tab except Compare took a single `annual_rate_percent`, so
every one of them modelled a Singapore package as though its promotional rate
ran for twenty-five years. The Payment tab then assessed TDSR on that rate and
passed borrowers a bank would decline (MAS Notice 645 para 6(b) assesses on
the *thereafter* rate). Nothing looked broken.

### Carry what a figure assumed, next to the figure

A calculated amount and the assumption it rests on travel together, and both
are decided in Rust. `RateType::floating_base` answers "was anything held
still to produce this?"; `Loan` carries the answer through the builder for
the same reason it carries `Reversion`, and `Report` prints it.

The failure it prevents is quiet. A Singapore package is quoted over 3M SORA,
which MAS publishes and this app has no feed for, so the instalment, the
totals and the twenty-five year schedule are all exact *given* one number
nobody here controls. Rendered without that sentence, a page of exact figures
reads as a quotation rather than a projection -- and the closer the document
gets to the shape of a real disclosure, the more convincingly it reads that
way. MAS makes a bank admit the same thing on a Notice 632A fact sheet.

Note that the disclosure changes no arithmetic. `report.rs` has a test
asserting exactly that: the bands, the stress rows and the totals are
identical whether or not the base is declared to float. If disclosing
something starts changing a number, the model has been mixed up with the
copy.

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
