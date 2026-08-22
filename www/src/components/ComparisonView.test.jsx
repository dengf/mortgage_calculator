import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import ComparisonView from './ComparisonView';
import { I18nProvider } from '../i18n';
import { renderControlled } from '../test/controlled';
import { scenarioBindings } from '../test/wasm';

const preset = (label, rate, term) => ({
  label,
  rate_type: { kind: 'fixed', rate_percent: rate },
  term_years: term,
});

function mockWasm(presets) {
  return {
    get_common_rate_presets: vi.fn(() => presets),
    calculate_comparison: vi.fn(() => ({ rows: [], error: null })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
  };
}

// The seeding used to index straight into loaded[0] and loaded[2]. Any
// preset list shorter than three threw a TypeError during the mount effect
// and unmounted the whole Compare tab -- not just the seeded rows. These
// cover the short and empty lists rather than only the happy path the Rust
// side happens to return today.
describe('ComparisonView preset seeding', () => {
  it('seeds two contrasting entries from a full preset list', async () => {
    render(
      <ComparisonView
        wasmModule={mockWasm([
          preset('30-Year Fixed', 6.5, 30),
          preset('20-Year Fixed', 6.25, 20),
          preset('15-Year Fixed', 6.0, 15),
        ])}
        region="US"
      />,
    );

    expect(await screen.findByDisplayValue('30-Year Fixed')).toBeInTheDocument();
    expect(screen.getByDisplayValue('15-Year Fixed')).toBeInTheDocument();
  });

  it('survives a preset list shorter than the indexes it wants', async () => {
    render(
      <ComparisonView
        wasmModule={mockWasm([preset('30-Year Fixed', 6.5, 30), preset('15-Year Fixed', 6.0, 15)])}
        region="US"
      />,
    );

    expect(await screen.findByDisplayValue('30-Year Fixed')).toBeInTheDocument();
    expect(screen.getByDisplayValue('15-Year Fixed')).toBeInTheDocument();
  });

  it('survives a single preset', async () => {
    render(
      <ComparisonView wasmModule={mockWasm([preset('30-Year Fixed', 6.5, 30)])} region="US" />,
    );

    expect(await screen.findByDisplayValue('30-Year Fixed')).toBeInTheDocument();
  });

  it('renders without crashing when no presets come back at all', async () => {
    render(<ComparisonView wasmModule={mockWasm([])} region="US" />);

    // The tab still mounts; there is simply nothing seeded to compare yet.
    expect(await screen.findByText('Saved scenarios')).toBeInTheDocument();
  });

  it('renders when the binding is missing entirely, as on a stale wasm build', async () => {
    const wasm = mockWasm([]);
    delete wasm.get_common_rate_presets;

    render(<ComparisonView wasmModule={wasm} region="US" />);

    expect(await screen.findByText('Saved scenarios')).toBeInTheDocument();
  });
});

describe('ComparisonView trade-off summary', () => {
  const rows = [
    {
      label: '30-Year Fixed',
      effective_rate_percent: 6.5,
      term_years: 30,
      payment: 2528.27,
      total_paid: 910177.2,
      total_interest: 510177.2,
    },
    {
      label: '15-Year Fixed',
      effective_rate_percent: 6.0,
      term_years: 15,
      payment: 3375.43,
      total_paid: 607577.4,
      total_interest: 207577.4,
    },
  ];

  // Which row wins and by how much is mortgage-calc's answer, tested in
  // crates/mortgage-calc/src/comparison.rs. These assert only that the
  // component renders the verdict it was handed, in the reader's language --
  // recomputing it here would be the duplication this change removed.
  const split = {
    cheapest_payment: 0,
    cheapest_interest: 1,
    cheapest_total_paid: 1,
    kind: 'split',
    cheaper: 1,
    lighter: 0,
    payment_delta: 847.16,
    interest_delta: 302599.8,
  };

  const outright = {
    cheapest_payment: 1,
    cheapest_interest: 1,
    cheapest_total_paid: 1,
    kind: 'outright',
    cheaper: 1,
    lighter: 1,
    payment_delta: 0,
    interest_delta: 0,
  };

  // Entries seed from presets, and nothing is computed without them.
  const wasmWith = (comparisonRows, verdict = null) => ({
    get_common_rate_presets: vi.fn(() => [
      preset('30-Year Fixed', 6.5, 30),
      preset('20-Year Fixed', 6.25, 20),
      preset('15-Year Fixed', 6.0, 15),
    ]),
    calculate_comparison: vi.fn(() => ({ rows: comparisonRows, verdict, error: null })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
  });

  it('states the difference, which is the question a comparison is asked', async () => {
    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasmWith(rows, split)} region="US" />
      </I18nProvider>,
    );
    // The delta between the two, not just two columns of figures.
    expect(await screen.findByText(/costs \$847\.16 more each month/)).toBeInTheDocument();
    expect(screen.getByText(/saves \$302,599\.80 in interest/)).toBeInTheDocument();
  });

  it('says so when one option wins outright', async () => {
    const dominant = [rows[0], { ...rows[1], payment: 100, total_paid: 1, total_interest: 1 }];
    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasmWith(dominant, outright)} region="US" />
      </I18nProvider>,
    );
    expect(await screen.findByText(/wins on both/)).toBeInTheDocument();
  });

  it('offers no trade-off line for a single scenario', async () => {
    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasmWith([rows[0]])} region="US" />
      </I18nProvider>,
    );
    await screen.findByText('30-Year Fixed');
    expect(document.querySelector('.cmp-tradeoff')).toBeNull();
  });
});

// Loading a saved comparison called setPrincipal/setFrequency, two setters
// that stopped existing when price and deposit moved into the shared
// scenario. Every Load click threw a ReferenceError and took the tab down.
// This shipped.
describe('ComparisonView saved scenarios', () => {
  const savedWasm = (inputs) => ({
    get_common_rate_presets: vi.fn(() => [preset('30-Year Fixed', 6.5, 30)]),
    calculate_comparison: vi.fn(() => ({ rows: [], error: null })),
    list_scenarios: vi.fn(async () => ({
      scenarios: [{ id: 's1', name: 'Saved run', created_at: Date.now() }],
      error: null,
    })),
    load_scenario: vi.fn(async () => ({
      scenario: { inputs_json: JSON.stringify(inputs) },
      error: null,
    })),
  });

  it('restores price and deposit from a saved comparison', async () => {
    const user = userEvent.setup();
    renderControlled(ComparisonView, {
      wasmModule: savedWasm({
        homePrice: 900000,
        downPayment: 180000,
        frequency: 'biweekly',
        entries: [],
      }),
      region: 'US',
    });

    await user.click(await screen.findByRole('button', { name: 'Load' }));

    expect(await screen.findByDisplayValue('900,000')).toBeInTheDocument();
    expect(screen.getByDisplayValue('180,000')).toBeInTheDocument();
  });

  it('restores a record saved before the scenario held a deposit', async () => {
    const user = userEvent.setup();
    renderControlled(ComparisonView, {
      wasmModule: savedWasm({ principal: 250000, frequency: 'monthly', entries: [] }),
      region: 'US',
    });

    await user.click(await screen.findByRole('button', { name: 'Load' }));

    // The split was never recorded, so the saved loan becomes the price and
    // the deposit is zero -- no figure invented on the user's behalf.
    expect(await screen.findByDisplayValue('250,000')).toBeInTheDocument();
    expect(screen.getByDisplayValue('0')).toBeInTheDocument();
  });
});

// Which floating index applies is a fact about the market the property is in.
// SGD loans reference SORA and have not referenced SIBOR since MAS
// discontinued it after 31 December 2024, so offering a Singapore buyer a
// SOFR or Prime quote is the wrong index, not a wording problem.
describe('ComparisonView rate presets', () => {
  const soraPreset = {
    label: 'Floating: 3M SORA + 0.50%',
    label_message: {
      code: 'preset.floating',
      params: { index: '3M SORA', spread: '0.50' },
      text: 'Floating: 3M SORA + 0.50%',
    },
    rate_type: { kind: 'floating', base_rate_percent: 1.12, spread_percent: 0.5 },
    term_years: 30,
  };

  const presetWasm = (presets) => ({
    get_common_rate_presets: vi.fn(() => presets),
    calculate_comparison: vi.fn(() => ({ rows: [], verdict: null, error: null })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
  });

  it('asks for the presets of the region being shopped in', async () => {
    const wasm = presetWasm([soraPreset]);
    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasm} region="SG" />
      </I18nProvider>,
    );

    await screen.findByDisplayValue('Floating: 3M SORA + 0.50%');
    expect(wasm.get_common_rate_presets).toHaveBeenCalledWith('SG');
  });

  it('names a preset in the reader language, not the English the core ships', async () => {
    render(
      <I18nProvider initialLocale="zh-Hans">
        <ComparisonView wasmModule={presetWasm([soraPreset])} region="SG" />
      </I18nProvider>,
    );

    // The index itself stays in Latin script -- a Singapore bank's own term
    // sheet says "3M SORA".
    expect(await screen.findByDisplayValue('浮动利率：3M SORA + 0.50%')).toBeInTheDocument();
    expect(screen.queryByDisplayValue(/Floating:/)).not.toBeInTheDocument();
  });

  it('falls back to the English rendering when there is no code', async () => {
    const legacy = { ...soraPreset, label_message: undefined };
    render(
      <I18nProvider initialLocale="zh-Hans">
        <ComparisonView wasmModule={presetWasm([legacy])} region="SG" />
      </I18nProvider>,
    );

    expect(await screen.findByDisplayValue('Floating: 3M SORA + 0.50%')).toBeInTheDocument();
  });

  it('reseeds when the market changes, rather than pricing SG rows at US rates', async () => {
    const usPreset = {
      label: '30-Year Fixed',
      label_message: { code: 'preset.fixed', params: { years: '30' }, text: '30-Year Fixed' },
      rate_type: { kind: 'fixed', rate_percent: 6.5 },
      term_years: 30,
    };
    const wasm = {
      get_common_rate_presets: vi.fn((region) => (region === 'SG' ? [soraPreset] : [usPreset])),
      calculate_comparison: vi.fn(() => ({ rows: [], verdict: null, error: null })),
      list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    };

    const { rerender } = render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasm} region="US" />
      </I18nProvider>,
    );
    expect(await screen.findByDisplayValue('30-Year Fixed')).toBeInTheDocument();

    rerender(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasm} region="SG" />
      </I18nProvider>,
    );

    // Carrying the US row over would keep computing a 6.5% 30-year fixed and
    // relabel it S$ -- a product Singapore does not offer, at several times
    // the market rate.
    expect(await screen.findByDisplayValue('Floating: 3M SORA + 0.50%')).toBeInTheDocument();
    expect(screen.queryByDisplayValue('30-Year Fixed')).not.toBeInTheDocument();
  });
});

// Saved comparisons store their rows verbatim, so a record written before a
// rate shape existed comes back missing that shape's fields.
describe('ComparisonView saved reverting rows', () => {
  const savedWasm = (inputs) => ({
    ...scenarioBindings(),
    get_common_rate_presets: vi.fn(() => []),
    calculate_comparison: vi.fn(() => ({ rows: [], verdict: null, error: null })),
    list_scenarios: vi.fn(async () => ({
      scenarios: [{ id: 's1', name: 'Saved run', created_at: Date.now() }],
      error: null,
    })),
    load_scenario: vi.fn(async () => ({
      scenario: { inputs_json: JSON.stringify(inputs) },
      error: null,
    })),
  });

  it('round-trips a reverting row', async () => {
    const user = userEvent.setup();
    const wasm = savedWasm({
      homePrice: 500000,
      downPayment: 100000,
      frequency: 'monthly',
      entries: [
        {
          label: 'My package',
          kind: 'reverting',
          baseRatePercent: 1.12,
          initialSpreadPercent: 0.35,
          initialYears: 3,
          thereafterSpreadPercent: 0.75,
          termYears: 25,
        },
      ],
    });
    renderControlled(ComparisonView, { wasmModule: wasm, region: 'SG' });

    await user.click(await screen.findByRole('button', { name: 'Load' }));

    expect(await screen.findByDisplayValue('My package')).toBeInTheDocument();
    expect(screen.getByDisplayValue('0.35')).toBeInTheDocument();
    expect(screen.getByDisplayValue('0.75')).toBeInTheDocument();
    expect(wasm.calculate_comparison).toHaveBeenLastCalledWith(
      expect.objectContaining({
        entries: [
          expect.objectContaining({
            rate_type: {
              kind: 'reverting',
              base_rate_percent: 1.12,
              initial_spread_percent: 0.35,
              initial_years: 3,
              thereafter_spread_percent: 0.75,
            },
          }),
        ],
      }),
    );
  });

  it('gives a row saved before reverting rates the fields it now needs', async () => {
    const user = userEvent.setup();
    const wasm = savedWasm({
      homePrice: 500000,
      downPayment: 100000,
      entries: [{ label: 'Old row', kind: 'fixed', ratePercent: 6.5, termYears: 30 }],
    });
    renderControlled(ComparisonView, { wasmModule: wasm, region: 'SG' });

    await user.click(await screen.findByRole('button', { name: 'Load' }));
    await screen.findByDisplayValue('Old row');

    // Switching an old row to the new shape must not send undefined spreads
    // across the boundary -- undefined arrives as a calculation, not an error.
    const toggles = document.querySelectorAll('.comparison-entry .kind-toggle');
    await user.click(toggles[1]);

    const sent = wasm.calculate_comparison.mock.lastCall[0].entries[0].rate_type;
    expect(sent.kind).toBe('reverting');
    for (const value of Object.values(sent)) {
      expect(value).toBeDefined();
    }
  });
});

// A row seeded from a preset is named from its own figures. Editing them used
// to leave the name behind, so a row could read "then + 0.60%" while
// computing 1.50% -- the label contradicting the numbers beside it.
describe('ComparisonView row names follow their figures', () => {
  const preset = {
    label: 'Floating: 3M SORA + 0.30% for 2 yr, then + 0.60%',
    label_message: {
      code: 'preset.reverting',
      params: { index: '3M SORA', initial: '0.30', years: '2', thereafter: '0.60' },
      text: '3M SORA + 0.30% for 2 yr, then + 0.60%',
    },
    index: '3M SORA',
    rate_type: {
      kind: 'reverting',
      base_rate_percent: 1.12,
      initial_spread_percent: 0.3,
      initial_years: 2,
      thereafter_spread_percent: 0.6,
    },
    term_years: 25,
  };

  const wasmWith = (describe_rate) => ({
    ...scenarioBindings(),
    get_common_rate_presets: vi.fn(() => [preset]),
    calculate_comparison: vi.fn(() => ({ rows: [], verdict: null, error: null })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    describe_rate,
  });

  const thereafterInput = () =>
    [...document.querySelectorAll('.comparison-entry-field')]
      .find((f) => /Thereafter/.test(f.textContent))
      .querySelector('input');

  it('renames the row when a spread changes', async () => {
    const user = userEvent.setup();
    const describe_rate = vi.fn(({ rate_type }) => ({
      code: 'preset.reverting',
      params: {
        index: '3M SORA',
        initial: rate_type.initial_spread_percent.toFixed(2),
        years: String(rate_type.initial_years),
        thereafter: rate_type.thereafter_spread_percent.toFixed(2),
      },
      text: 'ignored',
    }));

    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasmWith(describe_rate)} region="SG" />
      </I18nProvider>,
    );

    await screen.findByDisplayValue(/then \+ 0\.60%/);
    const field = thereafterInput();
    await user.clear(field);
    await user.type(field, '1.5');

    expect(await screen.findByDisplayValue(/then \+ 1\.50%/)).toBeInTheDocument();
    expect(screen.queryByDisplayValue(/then \+ 0\.60%/)).not.toBeInTheDocument();
  });

  it('leaves a name the user typed alone', async () => {
    const user = userEvent.setup();
    const describe_rate = vi.fn(() => ({
      code: 'preset.reverting',
      params: { index: '3M SORA', initial: '0.30', years: '2', thereafter: '9.99' },
      text: 'ignored',
    }));

    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasmWith(describe_rate)} region="SG" />
      </I18nProvider>,
    );

    const name = await screen.findByDisplayValue(/then \+ 0\.60%/);
    await user.clear(name);
    await user.type(name, 'DBS quote');

    const field = thereafterInput();
    await user.clear(field);
    await user.type(field, '1.5');

    // Their name, not ours.
    expect(screen.getByDisplayValue('DBS quote')).toBeInTheDocument();
  });
});
