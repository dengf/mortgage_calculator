import { describe, expect, it } from 'vitest';
import {
  DEFAULT_RATE,
  RATE_FIELDS,
  normalizeRate,
  rateFromPreset,
  rateValues,
  toRateTypeDto,
} from './rate';

// This module is the boundary contract for a quoted rate: what the forms
// edit on one side, what `RateTypeDto` deserializes on the other. Nothing
// here computes — a rate's effective value, its instalments and the rate a
// bank assesses it at all come back from Rust.

describe('the wire form of a rate', () => {
  it('sends only the fields the active shape uses', () => {
    // The form holds every field of every shape, so switching kind keeps
    // what was typed. Sending all of them would make serde reject the
    // untagged extras — and would describe a loan nobody quoted.
    expect(toRateTypeDto({ ...DEFAULT_RATE, kind: 'fixed', ratePercent: 6.5 })).toEqual({
      kind: 'fixed',
      rate_percent: 6.5,
    });
  });

  it('carries a package as four figures, not as its promotional rate', () => {
    expect(
      toRateTypeDto({
        kind: 'reverting',
        baseRatePercent: 1.12,
        initialSpreadPercent: 0.3,
        initialYears: 2,
        thereafterSpreadPercent: 0.6,
      }),
    ).toEqual({
      kind: 'reverting',
      base_rate_percent: 1.12,
      base_floats: true,
      initial_spread_percent: 0.3,
      initial_years: 2,
      thereafter_spread_percent: 0.6,
    });
  });

  it('coerces a field left as a typed string', () => {
    // Inputs hand back strings. serde wants an f64 and rejects `"6.5"`,
    // which used to surface as a raw parse error on the page.
    expect(toRateTypeDto({ kind: 'fixed', ratePercent: '6.5' })).toEqual({
      kind: 'fixed',
      rate_percent: 6.5,
    });
  });
});

describe('rates saved before a rate was a shape', () => {
  it('loads a bare number as the flat rate it was quoted at', () => {
    expect(normalizeRate(6.5)).toMatchObject({ kind: 'fixed', ratePercent: 6.5 });
  });

  it('leaves a shape alone', () => {
    expect(
      normalizeRate({ kind: 'floating', baseRatePercent: 3.63, spreadPercent: 2 }),
    ).toMatchObject({ kind: 'floating', baseRatePercent: 3.63, spreadPercent: 2 });
  });

  it('fills in the fields a shape does not use, so switching kind has values', () => {
    expect(normalizeRate({ kind: 'fixed', ratePercent: 6.5 }).thereafterSpreadPercent).toBe(
      DEFAULT_RATE.thereafterSpreadPercent,
    );
  });
});

describe('a preset read into a form', () => {
  it('keeps every part of a package editable', () => {
    // Every one of these is negotiated with the bank, so every one of them
    // has to be a field the buyer can change.
    const rate = rateFromPreset({
      rate_type: {
        kind: 'reverting',
        base_rate_percent: 1.12,
        base_floats: true,
        initial_spread_percent: 0.3,
        initial_years: 2,
        thereafter_spread_percent: 0.6,
      },
      term_years: 25,
    });

    expect(rateValues(rate)).toEqual([1.12, 0.3, 2, 0.6]);
  });

  it('reports the values a flat quote needs, and no others', () => {
    expect(rateValues({ kind: 'fixed', ratePercent: 6.5 })).toEqual([6.5]);
  });
});

describe('whether a step-up rests on something that moves', () => {
  it('assumes it does until told otherwise', () => {
    // Every package with this shape in the market it comes from is quoted
    // over 3M SORA. Defaulting the other way would silently drop the
    // caveat from every Singapore scenario saved before the field existed.
    expect(toRateTypeDto({ ...DEFAULT_RATE, kind: 'reverting' }).base_floats).toBe(true);
    expect(normalizeRate({ kind: 'reverting', baseRatePercent: 1.12 }).baseFloats).toBe(true);
  });

  it('carries the answer across when it is no', () => {
    const dto = toRateTypeDto({ ...DEFAULT_RATE, kind: 'reverting', baseFloats: false });
    expect(dto.base_floats).toBe(false);
  });

  it('is not asked of a shape that has no answer to give', () => {
    // A floating quote rests on a benchmark by construction and a fixed one
    // has no base at all. Sending the field would invite a caller to set it.
    expect(toRateTypeDto({ ...DEFAULT_RATE, kind: 'fixed' })).not.toHaveProperty('base_floats');
    expect(toRateTypeDto({ ...DEFAULT_RATE, kind: 'floating' })).not.toHaveProperty('base_floats');
  });

  it('reads a preset back with the basis it was quoted on', () => {
    const preset = {
      rate_type: {
        kind: 'reverting',
        base_rate_percent: 1.12,
        base_floats: true,
        initial_spread_percent: 0.3,
        initial_years: 2,
        thereafter_spread_percent: 0.6,
      },
    };
    expect(rateFromPreset(preset).baseFloats).toBe(true);
  });

  it('does not turn the toggle into an input field', () => {
    // RATE_FIELDS drives the number inputs on every form, and `rateValues`
    // gates the boundary call on them being filled. A boolean in that list
    // renders as a numeric box and blocks the tab when it is false.
    expect(RATE_FIELDS.reverting.map((f) => f.key)).not.toContain('baseFloats');
    expect(rateValues({ ...DEFAULT_RATE, kind: 'reverting', baseFloats: false })).not.toContain(
      false,
    );
  });
});
