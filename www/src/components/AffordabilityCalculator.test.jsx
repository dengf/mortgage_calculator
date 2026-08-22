import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AffordabilityCalculator from './AffordabilityCalculator';
import { I18nProvider } from '../i18n';

// Another panel with no tests. The US affordability model is the one a
// Singapore reader must never be shown -- it has no TDSR, no MSR, no LTV
// step-down and no ABSD in it -- so App routes SG to its own component. What
// this covers is that the US one behaves.

function mockWasm(overrides = {}) {
  return {
    calculate_affordability: vi.fn(() => ({
      max_monthly_housing_payment: 2800,
      max_principal_and_interest: 2200,
      max_loan_amount: 348000,
      max_home_price: 448000,
      front_end_dti_percent: 28,
      back_end_dti_percent: 36,
      error: null,
    })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    ...overrides,
  };
}

const show = (wasmModule, locale = 'en') =>
  render(
    <I18nProvider initialLocale={locale}>
      <AffordabilityCalculator wasmModule={wasmModule} region="US" />
    </I18nProvider>,
  );

describe('AffordabilityCalculator', () => {
  it('renders the ceiling the module computed', async () => {
    show(mockWasm());
    // Rounded to the nearest thousand: the figure comes from an assumed
    // ratio, not a quote, and cents would imply a precision it lacks.
    expect(await screen.findByText('$448,000')).toBeInTheDocument();
  });

  it('labels the property-tax rate per year in the reader language', async () => {
    // The suffix was the literal "%/yr", which read as English inside an
    // otherwise translated form.
    show(mockWasm(), 'zh-Hans');
    expect(await screen.findByText('%/年')).toBeInTheDocument();
  });

  it('passes the entered figures to the module', async () => {
    const user = userEvent.setup();
    const wasm = mockWasm();
    show(wasm);

    const income = await screen.findByDisplayValue('10,000');
    await user.clear(income);
    await user.type(income, '12000');

    expect(wasm.calculate_affordability).toHaveBeenLastCalledWith(
      expect.objectContaining({ gross_monthly_income: 12000 }),
    );
  });

  it('reports a failure from the module rather than a figure', async () => {
    show(
      mockWasm({
        calculate_affordability: vi.fn(() => ({
          error: 'Monthly income must be greater than zero (got 0).',
          error_message: { code: 'err.invalidIncome', params: { value: '0' } },
        })),
      }),
      'zh-Hans',
    );

    // Composed from the code, so the reader gets their own language and the
    // value is interpolated rather than left as "{value}".
    // Matched on the sentence, not just the noun -- the field label
    // contains 月收入 too.
    expect(await screen.findByText(/月收入必须大于零/)).toBeInTheDocument();
    expect(screen.queryByText(/\{value\}/)).not.toBeInTheDocument();
  });
});
