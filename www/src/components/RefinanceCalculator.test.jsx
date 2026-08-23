import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import RefinanceCalculator from './RefinanceCalculator';
import { I18nProvider } from '../i18n';

// This panel had no tests at all. Everything else untested this session
// turned out to be hiding something.

const duration = (months) => ({
  years: Math.floor(months / 12),
  months: months % 12,
  total_months: months,
  years_exact: months / 12,
  periods_per_year: 12,
});

function mockWasm(overrides = {}) {
  return {
    describe_duration: vi.fn(({ periods }) => duration(periods)),
    calculate_refinance: vi.fn(() => ({
      current_payment: 2216.04,
      new_payment: 1798.65,
      payment_savings: 417.39,
      break_even_periods: 15,
      remaining_interest_on_current_loan: 364812,
      total_interest_on_new_loan: 347514,
      lifetime_savings: 11298,
      error: null,
    })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    ...overrides,
  };
}

const show = (wasmModule, locale = 'en') =>
  render(
    <I18nProvider initialLocale={locale}>
      <RefinanceCalculator wasmModule={wasmModule} region="US" />
    </I18nProvider>,
  );

describe('RefinanceCalculator', () => {
  it('reports the break-even in time, not a payment count', async () => {
    show(mockWasm());
    expect(await screen.findByText('1 yr 3 mo')).toBeInTheDocument();
  });

  it('says a refinance never pays back, in the reader language', async () => {
    // Previously the literal string 'Never', so a Chinese reader was told
    // their refinance never breaks even in English.
    show(
      mockWasm({ calculate_refinance: vi.fn(() => ({ break_even_periods: null, error: null })) }),
      'zh-Hans',
    );
    expect(await screen.findByText('永不回本')).toBeInTheDocument();
  });

  it('warns that a fresh term lengthens the debt', async () => {
    // The default is 300 months remaining refinanced into 30 fresh years:
    // a lower payment and five more years of it. Both facts matter, and
    // "lifetime savings" alone hides the second.
    show(mockWasm());
    expect(await screen.findByText(/5 yr/)).toBeInTheDocument();
  });

  it('stays quiet when the new term does not extend the debt', async () => {
    const user = userEvent.setup();
    show(mockWasm());
    const newTerm = await screen.findByDisplayValue('30');
    await user.clear(newTerm);
    await user.type(newTerm, '20');

    // 20 years is 240 months against 300 remaining -- shorter, so there is
    // nothing to warn about.
    expect(document.querySelector('.refi-term-warning')).toBeNull();
  });

  it('holds its peace until every field is filled', async () => {
    const user = userEvent.setup();
    const wasm = mockWasm();
    show(wasm);
    const balance = await screen.findByDisplayValue('300,000');
    wasm.calculate_refinance.mockClear();
    await user.clear(balance);

    expect(wasm.calculate_refinance).not.toHaveBeenCalled();
  });
});

describe('RefinanceCalculator, refinancing into a package', () => {
  const SG_PRESET = {
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
  };

  function sgWasm(overrides = {}) {
    return mockWasm({
      get_common_rate_presets: vi.fn(() => [SG_PRESET]),
      calculate_refinance: vi.fn(() => ({
        current_payment: 2216.04,
        new_payment: 1798.65,
        new_payment_after_reversion: 1_950.12,
        payment_savings: 417.39,
        break_even_periods: 15,
        remaining_interest_on_current_loan: 364812,
        total_interest_on_new_loan: 347514,
        lifetime_savings: 11298,
        error: null,
      })),
      ...overrides,
    });
  }

  const showSg = (wasmModule) =>
    render(
      <I18nProvider initialLocale="en">
        <RefinanceCalculator wasmModule={wasmModule} region="SG" />
      </I18nProvider>,
    );

  it('quotes the new loan as a package the market actually sells', async () => {
    // Refinancing in Singapore means moving onto another package that steps
    // up after its own lock-in. A single "New rate" box could only describe
    // the promotional years of it.
    const wasmModule = sgWasm();
    showSg(wasmModule);

    await screen.findByText('S$1,798.65');
    expect(wasmModule.calculate_refinance).toHaveBeenLastCalledWith(
      expect.objectContaining({
        new_rate: expect.objectContaining({
          kind: 'reverting',
          thereafter_spread_percent: 0.6,
        }),
        new_term_years: 25,
      }),
    );
  });

  it('names the instalment the new loan steps up to', async () => {
    showSg(sgWasm());
    expect(await screen.findByText('then S$1,950.12')).toBeInTheDocument();
  });

  it('says the monthly saving is only for the lock-in', async () => {
    // "$417.39 saved each month" is true for two years of a twenty-five
    // year loan, and it is the number a switch gets sold on.
    showSg(sgWasm());
    expect(await screen.findByText('during the lock-in only')).toBeInTheDocument();
  });

  it('leaves a flat quote unqualified', async () => {
    show(mockWasm());
    await screen.findByText('$1,798.65');
    expect(screen.queryByText('during the lock-in only')).toBeNull();
  });
});
