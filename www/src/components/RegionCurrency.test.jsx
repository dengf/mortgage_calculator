import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import AmortizationSchedule from './AmortizationSchedule';
import AffordabilityCalculator from './AffordabilityCalculator';
import RefinanceCalculator from './RefinanceCalculator';
import ComparisonView from './ComparisonView';
import PaymentCalculator from './PaymentCalculator';

// Regression guard for a bug that shipped to production: the region toggle
// was added with the Singapore panel, but only PaymentCalculator was
// updated to consume it. The other four calculators kept their own
// formatter hardcoded to en-US/USD, so a Singapore user saw "S$" on the
// Payment tab and "$" on every other one — the app telling them their SGD
// loan was in US dollars. Every panel is checked here so the next one added
// can't quietly reintroduce it.

function mockWasm() {
  return {
    calculate_payment: vi.fn(() => ({
      payment: 2528.27,
      total_periods: 360,
      total_paid: 910177.2,
      total_interest: 510177.2,
      error: null,
    })),
    calculate_amortization_schedule: vi.fn(() => ({
      rows: [
        {
          period: 1,
          payment: 2528.27,
          extra_payment: 0,
          principal_portion: 361.6,
          interest_portion: 2166.67,
          remaining_balance: 399638.4,
        },
      ],
      error: null,
    })),
    calculate_extra_payment_impact: vi.fn(() => ({ error: null })),
    calculate_affordability: vi.fn(() => ({
      max_monthly_housing_payment: 3100,
      max_principal_and_interest: 2800,
      max_loan_amount: 442000,
      max_home_price: 502000,
      front_end_dti_percent: 31,
      back_end_dti_percent: 36,
      error: null,
    })),
    calculate_refinance: vi.fn(() => ({
      current_payment: 2200,
      new_payment: 1900,
      payment_savings: 300,
      break_even_periods: 20,
      remaining_interest_on_current_loan: 200000,
      total_interest_on_new_loan: 150000,
      lifetime_savings: 44000,
      error: null,
    })),
    calculate_comparison: vi.fn(() => ({ rows: [], error: null })),
    get_common_rate_presets: vi.fn(() => []),
    calculate_singapore: vi.fn(() => ({ warnings: [], error: null })),
    calculate_united_states: vi.fn(() => ({
      loan_type: 'Conforming',
      property_tax_rate_percent: 0.7,
      monthly_property_tax: 291.67,
      down_payment: 100000,
      down_payment_percent: 20,
      pmi_required: false,
      monthly_pmi: 0,
      monthly_piti: 2819.94,
      monthly_tax_savings: null,
      net_monthly_cost: null,
      error: null,
    })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
  };
}

const PANELS = [
  ['PaymentCalculator', PaymentCalculator],
  ['AmortizationSchedule', AmortizationSchedule],
  ['AffordabilityCalculator', AffordabilityCalculator],
  ['RefinanceCalculator', RefinanceCalculator],
  ['ComparisonView', ComparisonView],
];

describe('region-aware currency', () => {
  it.each(PANELS)('%s renders SGD, not USD, in the SG region', async (_name, Panel) => {
    const { container } = render(<Panel wasmModule={mockWasm()} region="SG" />);

    // Wait a tick for any effect-driven render (SavedScenarios loads on mount).
    await screen.findAllByText(/loan amount|income|balance|Saved scenarios/i);

    const text = container.textContent;
    const sgd = (text.match(/S\$/g) || []).length;
    // A bare "$" not preceded by "S" means a USD figure leaked through.
    const bareUsd = (text.match(/(^|[^S])\$/g) || []).length;

    expect(sgd).toBeGreaterThan(0);
    expect(bareUsd).toBe(0);
  });

  it.each(PANELS)('%s renders USD in the US region', async (_name, Panel) => {
    const { container } = render(<Panel wasmModule={mockWasm()} region="US" />);

    await screen.findAllByText(/loan amount|income|balance|Saved scenarios/i);

    expect(container.textContent).not.toMatch(/S\$/);
    expect(container.textContent).toMatch(/\$/);
  });
});
