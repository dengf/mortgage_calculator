import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AmortizationSchedule from './AmortizationSchedule';
import { renderControlled } from '../test/controlled';
import { scenarioBindings } from '../test/wasm';
import { DEFAULT_SCENARIO } from '../scenario';

// Same defect as ComparisonView's: `onLoad` called setters that stopped
// existing when price, rate and term moved into the shared scenario. Every
// Load click threw a ReferenceError and took the tab down. This shipped, and
// the tab had no test file at all to catch it.
function mockWasm(inputs) {
  return {
    ...scenarioBindings(),
    calculate_amortization_schedule: vi.fn(() => ({ rows: [], error: null })),
    calculate_extra_payment_impact: vi.fn(() => ({ error: null })),
    list_scenarios: vi.fn(async () => ({
      scenarios: [{ id: 's1', name: 'Saved run', created_at: Date.now() }],
      error: null,
    })),
    load_scenario: vi.fn(async () => ({
      scenario: { inputs_json: JSON.stringify(inputs) },
      error: null,
    })),
  };
}

describe('AmortizationSchedule saved scenarios', () => {
  it('restores price, deposit, rate and term from a saved run', async () => {
    const user = userEvent.setup();
    renderControlled(AmortizationSchedule, {
      wasmModule: mockWasm({
        homePrice: 900000,
        downPayment: 180000,
        rate: 5.25,
        termYears: 15,
        frequency: 'monthly',
        extraPayment: 250,
      }),
      region: 'US',
    });

    await user.click(await screen.findByRole('button', { name: 'Load' }));
    // The shared loan fields start folded away on this tab -- expand to
    // reach the inputs the restore is supposed to have populated.
    await user.click(screen.getByRole('button', { name: /Loan details/ }));

    expect(await screen.findByDisplayValue('900,000')).toBeInTheDocument();
    expect(screen.getByDisplayValue('180,000')).toBeInTheDocument();
    expect(screen.getByDisplayValue('5.25')).toBeInTheDocument();
    expect(screen.getByDisplayValue('15')).toBeInTheDocument();
  });

  it('restores a record saved before the scenario held a deposit', async () => {
    const user = userEvent.setup();
    renderControlled(AmortizationSchedule, {
      wasmModule: mockWasm({
        principal: 250000,
        rate: 6.5,
        termYears: 30,
        frequency: 'monthly',
        extraPayment: 0,
      }),
      region: 'US',
    });

    await user.click(await screen.findByRole('button', { name: 'Load' }));
    await user.click(screen.getByRole('button', { name: /Loan details/ }));

    // The split was never recorded, so the saved loan becomes the price and
    // the deposit is zero -- no figure invented on the user's behalf. Asserted
    // through the derived loan amount rather than the deposit input, since a
    // bare "0" also matches the extra-payment field.
    expect(await screen.findByDisplayValue('250,000')).toBeInTheDocument();
    expect(document.querySelector('.field-derived-value').textContent).toBe('$250,000.00');
  });
});

// The yearly view is what the tab shows by default, and it had no coverage
// at all. It used to be summed in the component with f64 reduces over the
// period rows; it now comes from the same call that produced them.
describe('AmortizationSchedule yearly summary', () => {
  const row = (period, balance) => ({
    period,
    payment: 1000,
    extra_payment: 0,
    principal_portion: 400,
    interest_portion: 600,
    remaining_balance: balance,
  });

  function scheduleWasm(schedule) {
    return {
      ...scenarioBindings(),
      calculate_amortization_schedule: vi.fn(() => schedule),
      calculate_extra_payment_impact: vi.fn(() => ({ error: null })),
      list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    };
  }

  it('renders the years the module returned', async () => {
    renderControlled(AmortizationSchedule, {
      wasmModule: scheduleWasm({
        rows: [row(1, 99600), row(2, 99200)],
        yearly: [
          { year: 1, paid: 12000, principal: 4800, interest: 7200, remaining_balance: 95200 },
          { year: 2, paid: 12000, principal: 4800, interest: 7200, remaining_balance: 90400 },
        ],
        error: null,
      }),
      region: 'US',
    });

    expect(await screen.findByText('$95,200.00')).toBeInTheDocument();
    expect(screen.getByText('$90,400.00')).toBeInTheDocument();
  });

  it('shows no yearly table when the module returned no years', async () => {
    renderControlled(AmortizationSchedule, {
      wasmModule: scheduleWasm({ rows: [], yearly: [], error: null }),
      region: 'US',
    });

    // Previously the component would have derived an empty list itself; now
    // an absent grouping must not render an empty table either.
    await userEvent.click(await screen.findByRole('button', { name: /Loan details/ }));
    expect(screen.getByDisplayValue('500,000')).toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
  });
});

// Moved here from Charts when the chart stopped deriving its own span: the
// axis must reflect the schedule actually plotted, not the nominal term. A
// 30-year loan paid off early used to reach zero under a "Year 30" label --
// mislabelling precisely the fact the user added extra payments to see.
describe('AmortizationSchedule balance chart', () => {
  const row = (period) => ({
    period,
    payment: 3028,
    extra_payment: 500,
    principal_portion: 2000,
    interest_portion: 1028,
    remaining_balance: 400000 - period * 2000,
  });

  it('labels the axis from the schedule plotted, not the term entered', async () => {
    const describe_duration = vi.fn(({ periods }) => ({
      years: Math.floor(periods / 12),
      months: periods % 12,
      total_months: periods,
      years_exact: periods / 12,
      periods_per_year: 12,
    }));

    renderControlled(AmortizationSchedule, {
      wasmModule: {
        ...scenarioBindings(),
        describe_duration,
        // 233 periods of a nominal 360-period term.
        calculate_amortization_schedule: vi.fn(() => ({
          rows: Array.from({ length: 233 }, (_, i) => row(i + 1)),
          yearly: [],
          error: null,
        })),
        calculate_extra_payment_impact: vi.fn(() => ({ error: null })),
        list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
      },
      region: 'US',
    });

    expect(await screen.findByText('Year 19.4')).toBeInTheDocument();
    expect(screen.queryByText('Year 30')).not.toBeInTheDocument();
    expect(describe_duration).toHaveBeenCalledWith({ periods: 233, frequency: 'monthly' });
  });
});

describe('AmortizationSchedule, on a package that steps up', () => {
  it('sends the package whole, so the schedule re-amortizes at the reset', async () => {
    // A bank reprices the instalment when the lock-in ends and re-amortizes
    // over what is left. Sending one rate produced a schedule that charged
    // the promotional rate for twenty-five years -- every row after year
    // two wrong, and wrong in the borrower's favour.
    const wasmModule = mockWasm({});
    renderControlled(
      AmortizationSchedule,
      { wasmModule },
      {
        ...DEFAULT_SCENARIO,
        rate: {
          kind: 'reverting',
          baseRatePercent: 1.12,
          initialSpreadPercent: 0.3,
          initialYears: 2,
          thereafterSpreadPercent: 0.6,
        },
        termYears: 25,
      },
    );

    await waitFor(() => expect(wasmModule.calculate_amortization_schedule).toHaveBeenCalled());
    expect(wasmModule.calculate_amortization_schedule).toHaveBeenLastCalledWith(
      expect.objectContaining({
        loan: expect.objectContaining({
          rate: {
            kind: 'reverting',
            base_rate_percent: 1.12,
            base_floats: true,
            initial_spread_percent: 0.3,
            initial_years: 2,
            thereafter_spread_percent: 0.6,
          },
          term_years: 25,
        }),
      }),
    );
  });
});
