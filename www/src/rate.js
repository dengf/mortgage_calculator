/**
 * The one description of a quoted rate the whole frontend uses.
 *
 * A rate used to be a number on every tab except Compare, which meant the
 * Payment, Amortization and Refinance tabs could only describe a rate that
 * holds for the whole term. No Singapore home loan does: every one of them
 * opens on a promotional spread over SORA and steps up after two or three
 * years. Those tabs were quoting the teaser and calling it the loan.
 *
 * Nothing here computes. The shape is carried to Rust as a `rate_type` and
 * every figure derived from it — the effective rate, the instalment before
 * and after the step-up, the rate a bank assesses servicing at — comes back
 * from the core. See CLAUDE.md.
 */

/** Every field of every rate shape, so switching kind keeps what was typed
 *  rather than discarding it. Only the active kind's fields are sent. */
export const DEFAULT_RATE = {
  kind: 'fixed',
  ratePercent: 6.5,
  baseRatePercent: 4.3,
  spreadPercent: 2,
  initialSpreadPercent: 0.3,
  initialYears: 2,
  thereafterSpreadPercent: 0.6,
  // A step-up is assumed to be quoted over a benchmark until the user says
  // otherwise, because in the market this shape comes from it always is —
  // every SGD package is quoted over 3M SORA. The two states differ only in
  // what the reader is told, and the cost of being wrong is asymmetric: an
  // unnecessary caveat wastes a sentence, a missing one lets a projection
  // read as a quotation.
  baseFloats: true,
};

/**
 * Which inputs each shape is made of, in the order they're read.
 *
 * One list, rendered by both the panel forms and the compact Compare rows —
 * so a shape can't grow a field on one tab and not the other. Labels are
 * catalog keys, not sentences.
 */
// `mortgage-calc` rejects a combined rate above 100% (see `MAX_ANNUAL_RATE`
// in `crates/mortgage-calc/src/loan.rs`) -- that is the authoritative check.
// This mirrors it on each percent field only so a fat-fingered "999" is
// caught by the input itself rather than round-tripping to Rust first.
const MAX_RATE_PERCENT = 100;

export const RATE_FIELDS = {
  fixed: [{ key: 'ratePercent', label: 'rate.rate', unit: 'rate.percent', max: MAX_RATE_PERCENT }],
  floating: [
    { key: 'baseRatePercent', label: 'rate.base', unit: 'rate.percent', max: MAX_RATE_PERCENT },
    { key: 'spreadPercent', label: 'rate.spread', unit: 'rate.percent', max: MAX_RATE_PERCENT },
  ],
  // Every field of a Singapore package is editable, because every one of
  // them is negotiated: the index, the promotional spread, how long it
  // lasts, and the spread it steps up to. The last decides most of the
  // interest and is the one buyers are least often shown.
  reverting: [
    { key: 'baseRatePercent', label: 'rate.base', unit: 'rate.percent', max: MAX_RATE_PERCENT },
    {
      key: 'initialSpreadPercent',
      label: 'rate.initialSpread',
      unit: 'rate.percent',
      max: MAX_RATE_PERCENT,
    },
    { key: 'initialYears', label: 'rate.lockIn', unit: 'rate.yrs', min: 0 },
    {
      key: 'thereafterSpreadPercent',
      label: 'rate.thereafterSpread',
      unit: 'rate.percent',
      max: MAX_RATE_PERCENT,
    },
  ],
};

export const RATE_KINDS = ['fixed', 'reverting', 'floating'];

/** The wire form: exactly the fields the active kind uses, named as the
 *  `RateTypeDto` variant expects them. */
export function toRateTypeDto(rate) {
  const shape = normalizeRate(rate);
  switch (shape.kind) {
    case 'floating':
      return {
        kind: 'floating',
        base_rate_percent: Number(shape.baseRatePercent),
        spread_percent: Number(shape.spreadPercent),
      };
    case 'reverting':
      return {
        kind: 'reverting',
        base_rate_percent: Number(shape.baseRatePercent),
        base_floats: Boolean(shape.baseFloats),
        initial_spread_percent: Number(shape.initialSpreadPercent),
        initial_years: Number(shape.initialYears),
        thereafter_spread_percent: Number(shape.thereafterSpreadPercent),
      };
    default:
      return { kind: 'fixed', rate_percent: Number(shape.ratePercent) };
  }
}

/**
 * Reads a rate back out of anything that might hold one.
 *
 * A saved scenario from before this existed stores `rate: 6.5`. That record
 * describes a real loan the user entered and still loads, as the flat rate
 * it was.
 */
export function normalizeRate(value) {
  if (value == null) return DEFAULT_RATE;
  if (typeof value === 'number' || typeof value === 'string') {
    return { ...DEFAULT_RATE, kind: 'fixed', ratePercent: value };
  }
  return { ...DEFAULT_RATE, ...value };
}

/** The rate a preset quotes, in the shape a form edits. */
export function rateFromPreset(preset) {
  const rate = preset?.rate_type;
  if (!rate) return DEFAULT_RATE;
  return {
    ...DEFAULT_RATE,
    kind: rate.kind,
    ...(rate.kind === 'fixed' && { ratePercent: rate.rate_percent }),
    ...(rate.kind === 'floating' && {
      baseRatePercent: rate.base_rate_percent,
      spreadPercent: rate.spread_percent,
    }),
    ...(rate.kind === 'reverting' && {
      baseRatePercent: rate.base_rate_percent,
      baseFloats: rate.base_floats ?? true,
      initialSpreadPercent: rate.initial_spread_percent,
      initialYears: rate.initial_years,
      thereafterSpreadPercent: rate.thereafter_spread_percent,
    }),
  };
}

/** The values a tab must have before it can ask the core anything — the
 *  active kind's fields, so a blank one never crosses the boundary. */
export function rateValues(rate) {
  const shape = normalizeRate(rate);
  return (RATE_FIELDS[shape.kind] ?? RATE_FIELDS.fixed).map((field) => shape[field.key]);
}
