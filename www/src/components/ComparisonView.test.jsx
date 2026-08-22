import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import ComparisonView from './ComparisonView';
import { I18nProvider } from '../i18n';

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
    render(<ComparisonView wasmModule={mockWasm([preset('30-Year Fixed', 6.5, 30)])} region="US" />);

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
    { label: '30-Year Fixed', effective_rate_percent: 6.5, term_years: 30, payment: 2528.27, total_paid: 910177.2, total_interest: 510177.2 },
    { label: '15-Year Fixed', effective_rate_percent: 6.0, term_years: 15, payment: 3375.43, total_paid: 607577.4, total_interest: 207577.4 },
  ];

  // Entries seed from presets, and nothing is computed without them.
  const wasmWith = (comparisonRows) => ({
    get_common_rate_presets: vi.fn(() => [
      preset('30-Year Fixed', 6.5, 30),
      preset('20-Year Fixed', 6.25, 20),
      preset('15-Year Fixed', 6.0, 15),
    ]),
    calculate_comparison: vi.fn(() => ({ rows: comparisonRows, error: null })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
  });

  it('states the difference, which is the question a comparison is asked', async () => {
    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasmWith(rows)} region="US" />
      </I18nProvider>,
    );
    // The delta between the two, not just two columns of figures.
    expect(await screen.findByText(/costs \$847\.16 more each month/)).toBeInTheDocument();
    expect(screen.getByText(/saves \$302,599\.80 in interest/)).toBeInTheDocument();
  });

  it('says so when one option wins outright', async () => {
    const dominant = [
      rows[0],
      { ...rows[1], payment: 100, total_paid: 1, total_interest: 1 },
    ];
    render(
      <I18nProvider initialLocale="en">
        <ComparisonView wasmModule={wasmWith(dominant)} region="US" />
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
