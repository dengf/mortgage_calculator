import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { AppShell } from './App';
import { I18nProvider } from './i18n';
import { scenarioBindings } from './test/wasm';

function mockWasm() {
  return {
    ...scenarioBindings(),
    calculate_payment: vi.fn(() => ({
      payment: 2528.27,
      total_periods: 360,
      total_paid: 910177.2,
      total_interest: 510177.2,
      error: null,
    })),
    calculate_amortization_schedule: vi.fn(() => ({ rows: [], error: null })),
    calculate_united_states: vi.fn(() => ({
      loan_type: 'Conforming',
      property_tax_rate_percent: 0.7,
      monthly_property_tax: 291.67,
      down_payment: 100000,
      down_payment_percent: 20,
      pmi_required: false,
      monthly_pmi: 0,
      monthly_piti: 2819.94,
      error: null,
    })),
    get_common_rate_presets: vi.fn(() => []),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
  };
}

const renderApp = () =>
  render(
    <I18nProvider initialLocale="en">
      <AppShell wasmModule={mockWasm()} />
    </I18nProvider>,
  );

describe('shared scenario', () => {
  it('carries an edited loan across tabs instead of reverting to defaults', async () => {
    renderApp();

    const price = await screen.findByDisplayValue('500,000');
    await userEvent.clear(price);
    await userEvent.type(price, '750000');

    await userEvent.click(screen.getByRole('button', { name: 'Amortization' }));

    // The whole point of F4: the loan the user just dialled in is still here.
    await waitFor(() => expect(screen.getByDisplayValue('750,000')).toBeInTheDocument());

    await userEvent.click(screen.getByRole('button', { name: 'Compare' }));
    expect(screen.getByDisplayValue('750,000')).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Payment' }));
    expect(screen.getByDisplayValue('750,000')).toBeInTheDocument();
  });

  it('derives the loan from price and deposit, so the two cannot disagree', async () => {
    renderApp();
    // $500,000 price less $100,000 down. Previously price and loan were
    // independent inputs, and a $300k loan on a $250k home was accepted
    // without comment.
    await waitFor(() =>
      expect(document.querySelector('.field-derived-value').textContent).toBe('$400,000.00'),
    );

    const down = screen.getByDisplayValue('100,000');
    await userEvent.clear(down);
    await userEvent.type(down, '50000');

    await waitFor(() =>
      expect(document.querySelector('.field-derived-value').textContent).toBe('$450,000.00'),
    );
  });

  it('never derives a negative loan when the deposit exceeds the price', async () => {
    renderApp();
    const down = await screen.findByDisplayValue('100,000');
    await userEvent.clear(down);
    await userEvent.type(down, '900000');

    // Scoped to the derived-loan readout: $0.00 also appears as the PMI
    // figure, so a bare text query would pass without proving anything.
    await waitFor(() =>
      expect(document.querySelector('.field-derived-value').textContent).toBe('$0.00'),
    );
  });
});

describe('the loan a market opens on', () => {
  // Presets are the market's own, so what a "default rate" even is differs:
  // a US buyer's is a 30-year fixed quote, a Singapore buyer's is a 25-year
  // package that steps up after the lock-in.
  const PRESETS = {
    US: [
      {
        label: '30-Year Fixed',
        label_message: { code: 'preset.fixed', params: { years: '30' } },
        index: null,
        rate_type: { kind: 'fixed', rate_percent: 6.5 },
        term_years: 30,
      },
    ],
    SG: [
      {
        label: '3M SORA + 0.30% for 2 yr, then + 0.60%',
        label_message: { code: 'preset.reverting', params: {} },
        index: '3M SORA',
        rate_type: {
          kind: 'reverting',
          base_rate_percent: 1.12,
          initial_spread_percent: 0.3,
          initial_years: 2,
          thereafter_spread_percent: 0.6,
        },
        term_years: 25,
      },
    ],
  };

  function regionalWasm() {
    return {
      ...mockWasm(),
      get_common_rate_presets: vi.fn((region) => PRESETS[region] ?? []),
      calculate_singapore: vi.fn(() => ({ warnings: [], warning_codes: [] })),
    };
  }

  const renderWith = (wasmModule) =>
    render(
      <I18nProvider initialLocale="en">
        <AppShell wasmModule={wasmModule} />
      </I18nProvider>,
    );

  it('quotes a Singapore buyer a package, not a thirty-year fixed rate', async () => {
    // Switching to SG used to leave the loan on a flat 6.5% for 30 years:
    // several times the Singapore market, on a product no bank there sells,
    // relabelled S$ and priced as though it were real.
    const wasmModule = regionalWasm();
    renderWith(wasmModule);
    await waitFor(() => expect(wasmModule.calculate_payment).toHaveBeenCalled());

    await userEvent.click(screen.getByRole('button', { name: 'SG' }));

    await waitFor(() =>
      expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith(
        expect.objectContaining({
          rate: expect.objectContaining({ kind: 'reverting' }),
          term_years: 25,
        }),
      ),
    );
  });

  it('seeds from the region it detected, without waiting for a click', async () => {
    const wasmModule = regionalWasm();
    renderWith(wasmModule);

    await waitFor(() =>
      expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith(
        expect.objectContaining({ rate: { kind: 'fixed', rate_percent: 6.5 }, term_years: 30 }),
      ),
    );
  });
});
