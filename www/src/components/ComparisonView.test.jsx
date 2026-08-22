import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import ComparisonView from './ComparisonView';
import { I18nProvider } from '../i18n';
import { renderControlled } from '../test/controlled';

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
